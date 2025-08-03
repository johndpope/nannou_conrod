import { Page } from 'puppeteer';
import { TimelineHelpers } from '../helpers/timeline-helpers';

describe('Display Object Updates and Rendering', () => {
  let page: Page;
  let timeline: TimelineHelpers;

  beforeEach(async () => {
    page = await browser.newPage();
    await page.goto('http://localhost:8080');
    timeline = new TimelineHelpers(page);
    await timeline.waitForTimeline();
    
    // Set up test objects
    await page.evaluate(() => {
      (window as any).stage = {
        children: [],
        addChild: function(child: any) {
          this.children.push(child);
        },
        removeChild: function(child: any) {
          const index = this.children.indexOf(child);
          if (index > -1) {
            this.children.splice(index, 1);
          }
        },
        getChildByName: function(name: string) {
          return this.children.find((c: any) => c.name === name);
        }
      };
    });
  });

  afterEach(async () => {
    await page.close();
  });

  describe('TC5.1: Property Animation Sync', () => {
    it('should sync visual state with timeline position exactly', async () => {
      // Create object with tweened properties
      await page.evaluate(() => {
        const obj = {
          name: 'testObject',
          x: 0, y: 0,
          rotation: 0,
          scaleX: 1, scaleY: 1,
          alpha: 1
        };
        (window as any).stage.children.push(obj);
      });

      // Set up property animations via keyframes
      const keyframes = [
        { frame: 1, props: { x: 0, y: 0, rotation: 0, scaleX: 1, alpha: 1 } },
        { frame: 30, props: { x: 100, y: 50, rotation: 90, scaleX: 2, alpha: 0.5 } },
        { frame: 60, props: { x: 200, y: 100, rotation: 180, scaleX: 1, alpha: 1 } }
      ];

      for (const kf of keyframes) {
        await timeline.setFrameScript(kf.frame, `
          let obj = stage.getChildByName('testObject');
          obj.x = ${kf.props.x};
          obj.y = ${kf.props.y};
          obj.rotation = ${kf.props.rotation};
          obj.scaleX = ${kf.props.scaleX};
          obj.scaleY = ${kf.props.scaleX};
          obj.alpha = ${kf.props.alpha};
        `);
      }

      // Play animation
      await timeline.play();
      
      // Pause at various points and verify
      const checkPoints = [15, 30, 45, 60];
      
      for (const frame of checkPoints) {
        await timeline.waitForFrame(frame);
        await timeline.pause();
        
        const objState = await page.evaluate(() => {
          return (window as any).stage.getChildByName('testObject');
        });
        
        // Verify properties match expected interpolation
        if (frame === 30) {
          expect(objState.x).toBe(100);
          expect(objState.rotation).toBe(90);
          expect(objState.alpha).toBe(0.5);
        } else if (frame === 60) {
          expect(objState.x).toBe(200);
          expect(objState.rotation).toBe(180);
          expect(objState.alpha).toBe(1);
        }
        
        await timeline.play();
      }
    });

    it('should maintain smooth interpolation between keyframes', async () => {
      // Monitor interpolation smoothness
      const samples: any[] = [];
      
      await timeline.play();
      
      // Sample object state frequently
      for (let i = 0; i < 30; i++) {
        await timeline.waitForAnimation(33); // Sample at ~30Hz
        
        const state = await page.evaluate(() => {
          const obj = (window as any).stage.getChildByName('testObject');
          return obj ? { x: obj.x, y: obj.y, frame: (window as any).currentFrame } : null;
        });
        
        if (state) samples.push(state);
      }
      
      // Verify smooth progression (no jumps)
      for (let i = 1; i < samples.length; i++) {
        const deltaX = Math.abs(samples[i].x - samples[i-1].x);
        const deltaFrame = samples[i].frame - samples[i-1].frame;
        
        // Position change should be proportional to frame change
        if (deltaFrame > 0) {
          const changeRate = deltaX / deltaFrame;
          expect(changeRate).toBeLessThan(10); // No sudden jumps
        }
      }
    });
  });

  describe('TC5.2: Dynamic Object Creation/Destruction', () => {
    it('should create and destroy objects at correct frames', async () => {
      // Frame 1: Create objects
      await timeline.setFrameScript(1, `
        for (let i = 0; i < 10; i++) {
          let circle = {
            type: 'Shape',
            name: 'dynamic_' + i,
            x: i * 50,
            y: 100,
            radius: 20
          };
          stage.addChild(circle);
        }
        trace("Created " + stage.children.length + " objects");
      `);

      // Frame 60: Remove objects
      await timeline.setFrameScript(60, `
        let removed = 0;
        for (let i = 0; i < 10; i++) {
          let obj = stage.getChildByName('dynamic_' + i);
          if (obj) {
            stage.removeChild(obj);
            removed++;
          }
        }
        trace("Removed " + removed + " objects");
      `);

      // Play through creation
      await timeline.play();
      await timeline.waitForFrame(30);
      await timeline.pause();

      // Verify objects created
      let objectCount = await page.evaluate(() => {
        return (window as any).stage.children.length;
      });
      expect(objectCount).toBe(10);

      // Continue to destruction
      await timeline.play();
      await timeline.waitForFrame(70);
      await timeline.pause();

      // Verify objects removed
      objectCount = await page.evaluate(() => {
        return (window as any).stage.children.length;
      });
      expect(objectCount).toBe(0);

      // Test scrubbing maintains state
      await timeline.seekToFrame(30);
      objectCount = await page.evaluate(() => {
        return (window as any).stage.children.length;
      });
      expect(objectCount).toBe(10);

      await timeline.seekToFrame(70);
      objectCount = await page.evaluate(() => {
        return (window as any).stage.children.length;
      });
      expect(objectCount).toBe(0);
    });

    it('should handle memory properly with repeated creation/destruction', async () => {
      // Get initial memory
      const initialMemory = await timeline.getMemoryUsage();

      // Create/destroy cycle script
      await timeline.setFrameScript(1, `
        // Clear any existing
        stage.children = [];
        
        // Create new batch
        for (let i = 0; i < 100; i++) {
          stage.addChild({
            name: 'temp_' + i,
            data: new Array(1000).fill(0) // Some memory usage
          });
        }
      `);

      // Cycle through multiple times
      for (let cycle = 0; cycle < 5; cycle++) {
        await timeline.seekToFrame(1);
        await timeline.waitForAnimation(100);
      }

      // Check memory hasn't grown excessively
      const finalMemory = await timeline.getMemoryUsage();
      const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
      
      // Should not leak more than 10MB
      expect(memoryGrowth).toBeLessThan(10 * 1024 * 1024);
    });
  });

  describe('TC5.3: Graphics Rendering Update', () => {
    it('should update graphics every frame', async () => {
      // Create shape with graphics
      await page.evaluate(() => {
        const shape = {
          name: 'canvas',
          graphics: {
            commands: [],
            clear: function() { this.commands = []; },
            beginFill: function(color: number, alpha: number) {
              this.commands.push({ type: 'beginFill', color, alpha });
            },
            drawCircle: function(x: number, y: number, radius: number) {
              this.commands.push({ type: 'drawCircle', x, y, radius });
            }
          }
        };
        (window as any).stage.addChild(shape);
      });

      // Animated graphics script
      await timeline.setFrameScript(1, `
        let shape = stage.getChildByName("canvas");
        let frame = currentFrame || 1;
        
        shape.graphics.clear();
        shape.graphics.beginFill(0xFF0000, 1.0);
        shape.graphics.drawCircle(
          Math.sin(frame * 0.1) * 100 + 200,
          Math.cos(frame * 0.1) * 100 + 200,
          50
        );
        
        global.lastDrawFrame = frame;
      `);

      // Play and sample graphics updates
      await timeline.play();
      
      const graphicsStates = [];
      for (let i = 0; i < 10; i++) {
        await timeline.waitForAnimation(100);
        
        const state = await page.evaluate(() => {
          const shape = (window as any).stage.getChildByName("canvas");
          const lastCommand = shape.graphics.commands[shape.graphics.commands.length - 1];
          return {
            frame: (window as any).global?.lastDrawFrame,
            x: lastCommand?.x,
            y: lastCommand?.y
          };
        });
        
        graphicsStates.push(state);
      }

      // Verify graphics updated each sample
      for (let i = 1; i < graphicsStates.length; i++) {
        expect(graphicsStates[i].frame).toBeGreaterThan(graphicsStates[i-1].frame);
        expect(graphicsStates[i].x).not.toBe(graphicsStates[i-1].x);
        expect(graphicsStates[i].y).not.toBe(graphicsStates[i-1].y);
      }

      // Verify maintains 60 FPS with graphics updates
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(58);
    });

    it('should handle complex paths without artifacts', async () => {
      // Complex path drawing
      await timeline.setFrameScript(1, `
        let shape = stage.getChildByName("canvas");
        shape.graphics.clear();
        
        // Draw complex star path
        let points = 10;
        let outerRadius = 100;
        let innerRadius = 50;
        let centerX = 200;
        let centerY = 200;
        
        for (let i = 0; i < points * 2; i++) {
          let angle = (i / (points * 2)) * Math.PI * 2;
          let radius = i % 2 === 0 ? outerRadius : innerRadius;
          let x = centerX + Math.cos(angle) * radius;
          let y = centerY + Math.sin(angle) * radius;
          
          if (i === 0) {
            shape.graphics.moveTo(x, y);
          } else {
            shape.graphics.lineTo(x, y);
          }
        }
        
        shape.graphics.closePath();
        shape.graphics.fill();
      `);

      await timeline.seekToFrame(1);
      
      // Verify complex path rendered
      const pathCommands = await page.evaluate(() => {
        const shape = (window as any).stage.getChildByName("canvas");
        return shape.graphics.commands.length;
      });
      
      expect(pathCommands).toBeGreaterThan(20); // Complex path has many commands
    });
  });
});