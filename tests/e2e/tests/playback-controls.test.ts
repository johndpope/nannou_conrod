import { Page } from 'puppeteer';
import { TimelineHelpers } from '../helpers/timeline-helpers';

describe('Animation Playback Controls', () => {
  let page: Page;
  let timeline: TimelineHelpers;

  beforeEach(async () => {
    page = await browser.newPage();
    await page.goto('http://localhost:8080');
    timeline = new TimelineHelpers(page);
    await timeline.waitForTimeline();
    
    // Setup test animation
    await timeline.addLayer('Test Layer 1');
    await timeline.addLayer('Test Layer 2');
  });

  afterEach(async () => {
    await page.close();
  });

  describe('TC2.1: Play/Pause Toggle', () => {
    it('should respond immediately to play/pause commands', async () => {
      // Click Play
      const playStartTime = Date.now();
      await timeline.play();
      const playResponseTime = Date.now() - playStartTime;
      
      expect(playResponseTime).toBeLessThan(50);
      
      // Wait 2 seconds
      await timeline.waitForAnimation(2000);
      
      // Click Pause
      const pauseStartTime = Date.now();
      await timeline.pause();
      const pauseResponseTime = Date.now() - pauseStartTime;
      
      expect(pauseResponseTime).toBeLessThan(50);
      
      // Note current frame
      const pausedFrame = await timeline.getCurrentFrame();
      expect(pausedFrame).toBeGreaterThanOrEqual(118);
      expect(pausedFrame).toBeLessThanOrEqual(122);
      
      // Wait a bit to ensure animation is truly paused
      await timeline.waitForAnimation(500);
      const stillPausedFrame = await timeline.getCurrentFrame();
      expect(stillPausedFrame).toBe(pausedFrame);
      
      // Resume playback
      await timeline.play();
      await timeline.waitForAnimation(1000);
      await timeline.pause();
      
      // Verify progression from pause point
      const resumedFrame = await timeline.getCurrentFrame();
      expect(resumedFrame).toBeGreaterThan(pausedFrame);
      expect(resumedFrame).toBeLessThanOrEqual(pausedFrame + 65);
    });

    it('should update UI button states correctly', async () => {
      // Check initial state
      let playButtonVisible = await page.$('[data-testid="play-button"]') !== null;
      let pauseButtonVisible = await page.$('[data-testid="pause-button"]') !== null;
      
      expect(playButtonVisible).toBe(true);
      expect(pauseButtonVisible).toBe(false);
      
      // Play animation
      await timeline.play();
      
      // Check button states after play
      playButtonVisible = await page.$('[data-testid="play-button"]') !== null;
      pauseButtonVisible = await page.$('[data-testid="pause-button"]') !== null;
      
      expect(playButtonVisible).toBe(false);
      expect(pauseButtonVisible).toBe(true);
    });
  });

  describe('TC2.2: Stop and Reset', () => {
    it('should reset animation to frame 1 on stop', async () => {
      // Set initial positions for objects
      await page.evaluate(() => {
        const obj1 = { x: 100, y: 100, rotation: 0 };
        const obj2 = { x: 200, y: 200, rotation: 0 };
        (window as any).stageObjects = [obj1, obj2];
      });

      // Play to frame 60
      await timeline.play();
      await timeline.waitForFrame(60);
      
      // Stop animation
      await timeline.stop();
      
      // Verify reset
      const currentFrame = await timeline.getCurrentFrame();
      expect(currentFrame).toBe(1);
      
      // Verify objects reset to initial positions
      const objectStates = await page.evaluate(() => {
        return (window as any).stageObjects;
      });
      
      expect(objectStates[0]).toEqual({ x: 100, y: 100, rotation: 0 });
      expect(objectStates[1]).toEqual({ x: 200, y: 200, rotation: 0 });
    });

    it('should execute frame 1 scripts on stop', async () => {
      // Add frame 1 script
      await timeline.setFrameScript(1, `
        trace('Frame 1 initialization');
        global.initialized = true;
      `);

      // Clear initialization flag
      await page.evaluate(() => {
        (window as any).global = { initialized: false };
      });

      // Play to frame 30 then stop
      await timeline.play();
      await timeline.waitForFrame(30);
      await timeline.stop();

      // Check if frame 1 script executed
      const initialized = await page.evaluate(() => {
        return (window as any).global?.initialized;
      });

      expect(initialized).toBe(true);

      // Check console for trace
      const consoleOutput = await timeline.getConsoleOutput();
      expect(consoleOutput).toContain('Frame 1 initialization');
    });
  });

  describe('TC2.3: Timeline Scrubbing', () => {
    it('should provide real-time preview during scrub', async () => {
      // Set up keyframes
      const keyframes = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
      for (const frame of keyframes) {
        await timeline.setFrameScript(frame, `
          stage.object1.x = ${frame * 5};
          stage.object1.rotation = ${frame * 3.6};
        `);
      }

      // Drag slowly from frame 1 to 100
      const dragStartTime = Date.now();
      await timeline.dragPlayhead(1, 100);
      const dragTime = Date.now() - dragStartTime;

      // Verify drag was smooth (took reasonable time)
      expect(dragTime).toBeGreaterThan(500); // At least 500ms for smooth drag
      expect(dragTime).toBeLessThan(2000); // But not too slow

      // Check final position
      const finalFrame = await timeline.getCurrentFrame();
      expect(finalFrame).toBe(100);

      // Test reverse scrubbing
      await timeline.dragPlayhead(100, 50);
      const reverseFrame = await timeline.getCurrentFrame();
      expect(reverseFrame).toBe(50);
    });

    it('should have fast seek time', async () => {
      // Test multiple seek operations
      const seekTimes: number[] = [];
      const testFrames = [20, 80, 150, 30, 180, 10];

      for (const frame of testFrames) {
        const seekTime = await timeline.measureSeekTime(frame);
        seekTimes.push(seekTime);
      }

      // All seeks should be fast
      seekTimes.forEach(time => {
        expect(time).toBeLessThan(20); // Less than 20ms per seek
      });

      // Average should be very fast
      const avgSeekTime = seekTimes.reduce((a, b) => a + b, 0) / seekTimes.length;
      expect(avgSeekTime).toBeLessThan(15);
    });
  });

  describe('TC2.4: Frame-by-Frame Navigation', () => {
    it('should step through frames instantly with arrow keys', async () => {
      // Start at frame 50
      await timeline.seekToFrame(50);

      // Test right arrow (next frame)
      const rightStartTime = Date.now();
      await page.keyboard.press('ArrowRight');
      const rightTime = Date.now() - rightStartTime;

      expect(rightTime).toBeLessThan(16); // Less than one frame at 60fps
      const frameAfterRight = await timeline.getCurrentFrame();
      expect(frameAfterRight).toBe(51);

      // Test left arrow (previous frame)
      const leftStartTime = Date.now();
      await page.keyboard.press('ArrowLeft');
      const leftTime = Date.now() - leftStartTime;

      expect(leftTime).toBeLessThan(16);
      const frameAfterLeft = await timeline.getCurrentFrame();
      expect(frameAfterLeft).toBe(50);

      // Test Shift+arrows for 10-frame jumps
      await page.keyboard.down('Shift');
      await page.keyboard.press('ArrowRight');
      await page.keyboard.up('Shift');

      const frameAfterJump = await timeline.getCurrentFrame();
      expect(frameAfterJump).toBe(60);
    });

    it('should handle boundary conditions correctly', async () => {
      // Test at beginning
      await timeline.seekToFrame(1);
      await page.keyboard.press('ArrowLeft');
      
      const frameAtStart = await timeline.getCurrentFrame();
      expect(frameAtStart).toBe(1); // Should stay at frame 1

      // Test at end (assuming 300 total frames)
      await timeline.seekToFrame(300);
      await page.keyboard.press('ArrowRight');
      
      const frameAtEnd = await timeline.getCurrentFrame();
      expect(frameAtEnd).toBe(300); // Should stay at last frame
    });
  });
});