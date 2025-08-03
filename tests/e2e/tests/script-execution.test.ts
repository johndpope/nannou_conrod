import { Page } from 'puppeteer';
import { TimelineHelpers } from '../helpers/timeline-helpers';

describe('Rhai Script Execution Sanity Checks', () => {
  let page: Page;
  let timeline: TimelineHelpers;

  beforeEach(async () => {
    page = await browser.newPage();
    await page.goto('http://localhost:8080');
    timeline = new TimelineHelpers(page);
    await timeline.waitForTimeline();
    
    // Clear console for clean test
    await page.evaluate(() => console.clear());
  });

  afterEach(async () => {
    await page.close();
  });

  describe('TC4.1: Frame Script Basic Execution', () => {
    it('should execute frame scripts at correct times', async () => {
      // Set up frame scripts
      const scriptFrames = [1, 15, 30, 45, 60];
      
      await timeline.setFrameScript(1, `
        trace("Frame 1 executed");
        stage.getChildByName("box1").x = 100;
      `);
      
      await timeline.setFrameScript(30, `
        trace("Frame 30 executed");
        let box = stage.getChildByName("box1");
        box.rotation += 45;
      `);
      
      await timeline.setFrameScript(60, `
        trace("Frame 60 executed");
      `);

      // Create test object
      await page.evaluate(() => {
        (window as any).stage = {
          children: {
            box1: { x: 0, y: 0, rotation: 0 }
          },
          getChildByName: function(name: string) {
            return this.children[name];
          }
        };
      });

      // Play animation
      await timeline.play();
      
      // Wait for scripts to execute
      await timeline.waitForFrame(60, 10000);
      await timeline.pause();

      // Verify console output
      const consoleOutput = await timeline.getConsoleOutput();
      expect(consoleOutput).toContain('Frame 1 executed');
      expect(consoleOutput).toContain('Frame 30 executed');
      expect(consoleOutput).toContain('Frame 60 executed');

      // Verify object transformations
      const boxState = await page.evaluate(() => {
        return (window as any).stage.children.box1;
      });

      expect(boxState.x).toBe(100);
      expect(boxState.rotation).toBe(45);
    });

    it('should execute scripts exactly once per frame', async () => {
      // Set up a counter script
      await timeline.setFrameScript(10, `
        if (!global.counter) global.counter = 0;
        global.counter++;
        trace("Counter: " + global.counter);
      `);

      // Play through frame 10 multiple times
      await timeline.play();
      await timeline.waitForFrame(20);
      await timeline.stop();
      
      await timeline.play();
      await timeline.waitForFrame(20);
      await timeline.stop();

      // Check counter
      const counter = await page.evaluate(() => {
        return (window as any).global?.counter;
      });

      expect(counter).toBe(2); // Should execute exactly twice
    });
  });

  describe('TC4.2: Script Performance Benchmarking', () => {
    it('should complete heavy computations within frame budget', async () => {
      // Heavy computation script
      const heavyScript = `
        let particles = [];
        for (let i = 0; i < 100; i++) {
          particles.push({
            x: Math.random() * 800,
            y: Math.random() * 600,
            vx: (Math.random() - 0.5) * 10,
            vy: (Math.random() - 0.5) * 10
          });
        }
        
        for (let p of particles) {
          p.x += p.vx;
          p.y += p.vy;
        }
        
        global.particleCount = particles.length;
      `;

      await timeline.setFrameScript(1, heavyScript);

      // Measure execution time
      const startTime = performance.now();
      await timeline.seekToFrame(1);
      const executionTime = performance.now() - startTime;

      // Should complete quickly
      expect(executionTime).toBeLessThan(5);

      // Verify script executed
      const particleCount = await page.evaluate(() => {
        return (window as any).global?.particleCount;
      });
      expect(particleCount).toBe(100);

      // Test performance during continuous playback
      await timeline.play();
      const fpsStats = await timeline.measureFPS(3000);
      
      // Should maintain 60 FPS even with heavy script
      expect(fpsStats.avg).toBeGreaterThanOrEqual(58);
      expect(fpsStats.min).toBeGreaterThanOrEqual(55);
    });

    it('should not accumulate lag over time', async () => {
      // Script that does increasing work
      await timeline.setFrameScript(1, `
        if (!global.workItems) global.workItems = [];
        global.workItems.push({ data: new Array(100).fill(0) });
        
        // Process all items
        for (let item of global.workItems) {
          item.data = item.data.map(x => x + 1);
        }
      `);

      // Play for extended period
      await timeline.play();
      
      // Monitor FPS over time
      const fpsReadings = [];
      for (let i = 0; i < 10; i++) {
        await timeline.waitForAnimation(1000);
        const fps = await timeline.getFPS();
        fpsReadings.push(fps);
      }

      // FPS should remain stable (not decreasing)
      const firstHalf = fpsReadings.slice(0, 5).reduce((a, b) => a + b, 0) / 5;
      const secondHalf = fpsReadings.slice(5).reduce((a, b) => a + b, 0) / 5;
      
      expect(secondHalf).toBeGreaterThanOrEqual(firstHalf - 5);
    });
  });

  describe('TC4.3: Script Error Handling', () => {
    it('should handle undefined function errors gracefully', async () => {
      await timeline.setFrameScript(10, `
        trace("Before error");
        undefinedFunction(); // This should error
        trace("After error - should not execute");
      `);

      // Play through error frame
      await timeline.play();
      await timeline.waitForFrame(15);
      await timeline.pause();

      // Check console output
      const consoleOutput = await timeline.getConsoleOutput();
      expect(consoleOutput).toContain('Before error');
      expect(consoleOutput).not.toContain('After error - should not execute');
      
      // Should show error message
      const errorLogs = consoleOutput.filter(log => 
        log.includes('error') || log.includes('undefined')
      );
      expect(errorLogs.length).toBeGreaterThan(0);

      // Playback should continue
      const currentFrame = await timeline.getCurrentFrame();
      expect(currentFrame).toBe(15);
    });

    it('should handle null reference errors', async () => {
      await timeline.setFrameScript(5, `
        let obj = stage.getChildByName("nonexistent");
        obj.x = 10; // Null reference
      `);

      await timeline.play();
      await timeline.waitForFrame(10);
      
      // Should not crash
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(55);
    });

    it('should timeout infinite loops', async () => {
      await timeline.setFrameScript(20, `
        trace("Starting infinite loop");
        while(true) {
          // Infinite loop
        }
        trace("Should never reach here");
      `);

      const startFrame = await timeline.getCurrentFrame();
      
      // Play through infinite loop frame
      await timeline.play();
      await timeline.waitForAnimation(2000);
      
      // Should continue past the problematic frame
      const endFrame = await timeline.getCurrentFrame();
      expect(endFrame).toBeGreaterThan(20);

      // Check for timeout error
      const consoleOutput = await timeline.getConsoleOutput();
      const timeoutErrors = consoleOutput.filter(log => 
        log.includes('timeout') || log.includes('exceeded')
      );
      expect(timeoutErrors.length).toBeGreaterThan(0);
    });
  });

  describe('TC4.4: Cross-Frame State Persistence', () => {
    it('should persist global state across frames', async () => {
      // Frame 1: Initialize
      await timeline.setFrameScript(1, `
        global.counter = 0;
        global.data = { items: [] };
        trace("Initialized: counter=" + global.counter);
      `);

      // Frame 30: Update
      await timeline.setFrameScript(30, `
        global.counter += 1;
        global.data.items.push(currentFrame);
        trace("Frame 30: counter=" + global.counter);
      `);

      // Frame 60: Verify
      await timeline.setFrameScript(60, `
        trace("Final counter: " + global.counter);
        trace("Items collected: " + global.data.items.length);
      `);

      // Play through all frames
      await timeline.play();
      await timeline.waitForFrame(60);
      await timeline.pause();

      // Verify console output
      const consoleOutput = await timeline.getConsoleOutput();
      expect(consoleOutput).toContain('Initialized: counter=0');
      expect(consoleOutput).toContain('Frame 30: counter=1');
      expect(consoleOutput).toContain('Final counter: 1');
      expect(consoleOutput).toContain('Items collected: 1');

      // Verify state persisted
      const globalState = await page.evaluate(() => {
        return (window as any).global;
      });

      expect(globalState.counter).toBe(1);
      expect(globalState.data.items).toContain(30);
    });

    it('should reset state on stop', async () => {
      // Set up persistent state
      await timeline.setFrameScript(1, `
        global.resetTest = "initialized";
      `);

      await timeline.setFrameScript(30, `
        global.resetTest = "modified";
      `);

      // Play to frame 30
      await timeline.play();
      await timeline.waitForFrame(30);
      
      // Verify state modified
      let state = await page.evaluate(() => {
        return (window as any).global?.resetTest;
      });
      expect(state).toBe('modified');

      // Stop and replay
      await timeline.stop();
      await timeline.play();
      await timeline.waitForFrame(10);
      await timeline.pause();

      // State should be reset
      state = await page.evaluate(() => {
        return (window as any).global?.resetTest;
      });
      expect(state).toBe('initialized');
    });
  });
});