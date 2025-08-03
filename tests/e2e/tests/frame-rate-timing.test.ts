import { Page } from 'puppeteer';
import { TimelineHelpers } from '../helpers/timeline-helpers';

describe('Frame Rate and Timing Synchronization', () => {
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

  describe('TC1.1: Basic 60 FPS Playback', () => {
    it('should maintain consistent 60 FPS with 10 display objects', async () => {
      // Setup: Create animation with 10 objects
      for (let i = 0; i < 10; i++) {
        await timeline.addLayer(`Object ${i + 1}`);
      }

      // Start playback
      await timeline.play();

      // Monitor frame rate for 5 seconds
      const fpsStats = await timeline.measureFPS(5000);

      // Verify results
      expect(fpsStats.avg).toBeGreaterThanOrEqual(59);
      expect(fpsStats.avg).toBeLessThanOrEqual(61);
      expect(fpsStats.min).toBeGreaterThanOrEqual(58);
      
      // Check frame counter increments smoothly
      const frame1 = await timeline.getCurrentFrame();
      await timeline.waitForAnimation(1000);
      const frame2 = await timeline.getCurrentFrame();
      
      const framesDiff = frame2 - frame1;
      expect(framesDiff).toBeGreaterThanOrEqual(58);
      expect(framesDiff).toBeLessThanOrEqual(62);
    });
  });

  describe('TC1.2: Timeline-Engine Sync Verification', () => {
    it('should maintain perfect sync between timeline and engine', async () => {
      // Add frame scripts
      const testFrames = [1, 30, 60, 90];
      for (const frame of testFrames) {
        await timeline.setFrameScript(frame, `trace('Frame ${frame} executed');`);
      }

      // Clear console
      await page.evaluate(() => console.clear());

      // Play animation
      await timeline.play();

      // Wait for frame 90
      await timeline.waitForFrame(90, 10000);
      await timeline.pause();

      // Verify script execution
      const consoleOutput = await timeline.getConsoleOutput();
      
      expect(consoleOutput).toContain('Frame 1 executed');
      expect(consoleOutput).toContain('Frame 30 executed');
      expect(consoleOutput).toContain('Frame 60 executed');
      expect(consoleOutput).toContain('Frame 90 executed');

      // Verify current frame matches
      const currentFrame = await timeline.getCurrentFrame();
      expect(currentFrame).toBe(90);
    });

    it('should maintain sync after seeking and scrubbing', async () => {
      // Test seeking
      await timeline.seekToFrame(50);
      let frame = await timeline.getCurrentFrame();
      expect(frame).toBe(50);

      // Test scrubbing
      await timeline.dragPlayhead(50, 100);
      frame = await timeline.getCurrentFrame();
      expect(frame).toBe(100);

      // Play and verify sync maintained
      await timeline.play();
      await timeline.waitForAnimation(500);
      await timeline.pause();

      const finalFrame = await timeline.getCurrentFrame();
      expect(finalFrame).toBeGreaterThanOrEqual(125);
      expect(finalFrame).toBeLessThanOrEqual(135);
    });
  });

  describe('TC1.3: Variable Frame Rate Handling', () => {
    it('should adjust timing correctly when changing FPS', async () => {
      // Create 5-second animation at 24 FPS (120 frames)
      await page.evaluate(() => {
        // Set project to 24 FPS
        (window as any).timeline?.setFPS(24);
      });

      // Add test content
      await timeline.addLayer('Test Layer');

      // Play for 2 seconds at 24 FPS
      await timeline.play();
      await timeline.waitForAnimation(2000);
      await timeline.pause();

      const frame24fps = await timeline.getCurrentFrame();
      expect(frame24fps).toBeGreaterThanOrEqual(46);
      expect(frame24fps).toBeLessThanOrEqual(50);

      // Change to 30 FPS
      await page.evaluate(() => {
        (window as any).timeline?.setFPS(30);
      });

      // Reset and play for 2 seconds at 30 FPS
      await timeline.stop();
      await timeline.play();
      await timeline.waitForAnimation(2000);
      await timeline.pause();

      const frame30fps = await timeline.getCurrentFrame();
      expect(frame30fps).toBeGreaterThanOrEqual(58);
      expect(frame30fps).toBeLessThanOrEqual(62);

      // Change to 60 FPS
      await page.evaluate(() => {
        (window as any).timeline?.setFPS(60);
      });

      // Reset and play for 2 seconds at 60 FPS
      await timeline.stop();
      await timeline.play();
      await timeline.waitForAnimation(2000);
      await timeline.pause();

      const frame60fps = await timeline.getCurrentFrame();
      expect(frame60fps).toBeGreaterThanOrEqual(118);
      expect(frame60fps).toBeLessThanOrEqual(122);
    });
  });
});