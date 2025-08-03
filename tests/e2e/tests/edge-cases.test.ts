import { Page } from 'puppeteer';
import { TimelineHelpers } from '../helpers/timeline-helpers';

describe('Edge Cases and Error Scenarios', () => {
  let page: Page;
  let timeline: TimelineHelpers;

  beforeEach(async () => {
    page = await browser.newPage();
    await page.goto('http://localhost:8080');
    timeline = new TimelineHelpers(page);
    await timeline.waitForTimeline();
  });

  afterEach(async () => {
    await page.close();
  });

  describe('TC6.1: Rapid Play/Pause Cycling', () => {
    it('should handle rapid play/pause without crashes', async () => {
      // Rapid clicking test
      for (let i = 0; i < 20; i++) {
        if (i % 2 === 0) {
          await timeline.play();
        } else {
          await timeline.pause();
        }
        await timeline.waitForAnimation(50); // Small delay between clicks
      }

      // Verify UI still responsive
      const isResponsive = await page.evaluate(() => {
        return document.querySelector('[data-testid="timeline"]') !== null;
      });
      expect(isResponsive).toBe(true);

      // Check final state is consistent
      const frameAfterRapidClicks = await timeline.getCurrentFrame();
      expect(frameAfterRapidClicks).toBeGreaterThanOrEqual(1);

      // Verify can still play normally
      await timeline.stop();
      await timeline.play();
      await timeline.waitForAnimation(1000);
      const finalFrame = await timeline.getCurrentFrame();
      expect(finalFrame).toBeGreaterThan(50);
    });

    it('should handle spacebar spam (play toggle)', async () => {
      // Initial state
      const startFrame = await timeline.getCurrentFrame();

      // Spam spacebar
      for (let i = 0; i < 30; i++) {
        await page.keyboard.press('Space');
        await timeline.waitForAnimation(30);
      }

      // Should still be functional
      await timeline.stop();
      await timeline.play();
      await timeline.waitForAnimation(500);
      
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(55);
    });
  });

  describe('TC6.2: Maximum Layer Stress Test', () => {
    it('should handle 100 layers with acceptable performance', async () => {
      // Create 100 layers
      const startTime = Date.now();
      
      for (let i = 0; i < 100; i++) {
        await timeline.addLayer(`StressLayer${i}`);
        
        // Add simple animation to each layer
        await timeline.selectLayer(i);
        await timeline.setFrameScript(1, `
          let obj = {
            name: 'obj' + ${i},
            x: ${i * 5},
            y: 100,
            rotation: 0
          };
          stage.addChild(obj);
        `);
      }

      const setupTime = Date.now() - startTime;
      console.log(`Setup 100 layers in ${setupTime}ms`);

      // Play animation
      await timeline.play();

      // Monitor performance
      const fpsStats = await timeline.measureFPS(5000);
      
      // Should maintain at least 30 FPS
      expect(fpsStats.avg).toBeGreaterThanOrEqual(30);
      expect(fpsStats.min).toBeGreaterThanOrEqual(25);

      // Check memory usage
      const memory = await timeline.getMemoryUsage();
      expect(memory.heapUsed).toBeLessThan(500 * 1024 * 1024); // Less than 500MB

      // Verify all controls remain responsive
      await timeline.pause();
      const pauseResponseTime = Date.now();
      await timeline.play();
      const playResponseTime = Date.now() - pauseResponseTime;
      
      expect(playResponseTime).toBeLessThan(100);
    });

    it('should handle navigation with many layers', async () => {
      // Create 50 layers for navigation test
      for (let i = 0; i < 50; i++) {
        await timeline.addLayer(`NavLayer${i}`);
      }

      // Test various navigation operations
      const operations = [
        () => timeline.seekToFrame(100),
        () => timeline.dragPlayhead(100, 200),
        () => page.keyboard.press('ArrowRight'),
        () => page.keyboard.press('ArrowLeft'),
        () => timeline.selectLayer(25),
        () => timeline.toggleLayerVisibility(30)
      ];

      for (const op of operations) {
        const startTime = Date.now();
        await op();
        const opTime = Date.now() - startTime;
        
        // All operations should remain responsive
        expect(opTime).toBeLessThan(50);
      }
    });
  });

  describe('TC6.3: Script Memory Exhaustion', () => {
    it('should handle memory-intensive scripts gracefully', async () => {
      // Memory bomb script
      await timeline.setFrameScript(10, `
        try {
          let huge_array = [];
          for (let i = 0; i < 10000; i++) {
            huge_array.push({
              data: new Array(1000).fill(i)
            });
          }
          global.memoryTest = "should not complete";
        } catch (e) {
          trace("Memory limit reached: " + e.message);
          global.memoryTest = "failed gracefully";
        }
      `);

      // Play through the problematic frame
      await timeline.play();
      await timeline.waitForFrame(20);
      await timeline.pause();

      // Check if script was terminated gracefully
      const result = await page.evaluate(() => {
        return (window as any).global?.memoryTest;
      });

      expect(result).toBe("failed gracefully");

      // Engine should continue running
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(50);

      // Memory should be released
      await page.evaluate(() => {
        if ((global as any).gc) {
          (global as any).gc();
        }
      });

      const memory = await timeline.getMemoryUsage();
      expect(memory.heapUsed).toBeLessThan(200 * 1024 * 1024);
    });
  });

  describe('TC6.4: Concurrent Operation Conflicts', () => {
    it('should handle multiple simultaneous operations', async () => {
      // Start playback
      await timeline.play();

      // Perform multiple operations while playing
      const operations = Promise.all([
        timeline.seekToFrame(50),
        page.keyboard.down('Control').then(() => {
          page.keyboard.press('+');
          page.keyboard.up('Control');
        }),
        timeline.addLayer('ConcurrentLayer1'),
        timeline.addLayer('ConcurrentLayer2'),
        page.evaluate(() => {
          const grid = document.querySelector('[data-testid="frame-grid"]');
          if (grid) grid.scrollLeft = 200;
        })
      ]);

      // Wait for all operations
      await operations;

      // System should remain stable
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(45);

      // Verify operations completed
      const layers = await timeline.getLayers();
      expect(layers.length).toBeGreaterThanOrEqual(2);
    });

    it('should handle script editor operations during playback', async () => {
      // Start playback
      await timeline.play();

      // Open multiple script editors
      for (let i = 0; i < 3; i++) {
        await timeline.openScriptEditor();
        await timeline.waitForAnimation(100);
        
        // Type in editor while playing
        await page.keyboard.type(`// Script ${i}\ntrace("test");`);
      }

      // Close editors
      await page.keyboard.press('Escape');
      await page.keyboard.press('Escape');
      await page.keyboard.press('Escape');

      // Playback should continue uninterrupted
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(55);
    });

    it('should handle auto-save during playback', async () => {
      // Simulate auto-save trigger
      await timeline.play();
      
      // Trigger save operation
      await page.evaluate(() => {
        // Simulate auto-save
        const saveEvent = new Event('autosave');
        document.dispatchEvent(saveEvent);
      });

      // Wait for save to complete
      await timeline.waitForAnimation(500);

      // Playback should not be interrupted
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(55);

      // Data integrity check
      const currentFrame = await timeline.getCurrentFrame();
      await timeline.stop();
      await timeline.seekToFrame(currentFrame);
      
      const seekedFrame = await timeline.getCurrentFrame();
      expect(seekedFrame).toBe(currentFrame);
    });
  });

  describe('Performance Under Extreme Conditions', () => {
    it('should degrade gracefully under extreme load', async () => {
      // Create extreme load
      for (let i = 0; i < 50; i++) {
        await timeline.addLayer(`ExtremeLayer${i}`);
      }

      // Add heavy scripts to multiple frames
      for (let frame = 1; frame <= 10; frame++) {
        await timeline.setFrameScript(frame, `
          for (let i = 0; i < 100; i++) {
            let obj = {
              name: 'heavy' + i + '_' + ${frame},
              data: new Array(100).fill(Math.random())
            };
            stage.addChild(obj);
          }
        `);
      }

      // Start playback
      await timeline.play();

      // Monitor degradation
      const fpsReadings = [];
      for (let i = 0; i < 5; i++) {
        await timeline.waitForAnimation(1000);
        const fps = await timeline.getFPS();
        fpsReadings.push(fps);
      }

      // Should maintain minimum 15 FPS even under extreme load
      const minFps = Math.min(...fpsReadings);
      expect(minFps).toBeGreaterThanOrEqual(15);

      // UI should remain somewhat responsive
      const seekStartTime = Date.now();
      await timeline.seekToFrame(100);
      const seekTime = Date.now() - seekStartTime;
      
      expect(seekTime).toBeLessThan(200); // Degraded but still responsive
    });
  });
});