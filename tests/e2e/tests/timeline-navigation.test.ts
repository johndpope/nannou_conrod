import { Page } from 'puppeteer';
import { TimelineHelpers } from '../helpers/timeline-helpers';

describe('Timeline Scrolling and Navigation', () => {
  let page: Page;
  let timeline: TimelineHelpers;

  beforeEach(async () => {
    page = await browser.newPage();
    await page.goto('http://localhost:8080');
    timeline = new TimelineHelpers(page);
    await timeline.waitForTimeline();
    
    // Setup test with multiple layers
    for (let i = 0; i < 10; i++) {
      await timeline.addLayer(`Layer ${i + 1}`);
    }
  });

  afterEach(async () => {
    await page.close();
  });

  describe('TC3.1: Horizontal Scroll During Playback', () => {
    it('should auto-scroll to keep playhead visible', async () => {
      // Set timeline view to show only 100 frames
      await page.evaluate(() => {
        const timeline = document.querySelector('[data-testid="timeline"]');
        if (timeline) {
          (timeline as HTMLElement).style.width = '800px';
          (timeline as any).framesVisible = 100;
        }
      });

      // Start playback
      await timeline.play();

      // Wait for playhead to reach right edge (frame 100)
      await timeline.waitForFrame(100, 10000);

      // Check if timeline auto-scrolled
      const scrollPosition = await page.evaluate(() => {
        const frameGrid = document.querySelector('[data-testid="frame-grid"]');
        return frameGrid ? frameGrid.scrollLeft : 0;
      });

      expect(scrollPosition).toBeGreaterThan(0);

      // Continue playing and verify smooth auto-scroll
      await timeline.waitForFrame(150, 10000);

      const newScrollPosition = await page.evaluate(() => {
        const frameGrid = document.querySelector('[data-testid="frame-grid"]');
        return frameGrid ? frameGrid.scrollLeft : 0;
      });

      expect(newScrollPosition).toBeGreaterThan(scrollPosition);

      // Verify playback wasn't interrupted
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(58);
    });

    it('should allow manual scroll without affecting playback', async () => {
      // Start playback
      await timeline.play();

      // Get initial FPS
      const initialFPS = await timeline.getFPS();

      // Manually scroll timeline
      await page.evaluate(() => {
        const frameGrid = document.querySelector('[data-testid="frame-grid"]');
        if (frameGrid) {
          frameGrid.scrollLeft = 500;
        }
      });

      // Wait a bit
      await timeline.waitForAnimation(500);

      // Check FPS maintained
      const afterScrollFPS = await timeline.getFPS();
      expect(afterScrollFPS).toBeGreaterThanOrEqual(initialFPS - 2);

      // Verify playback continued
      const frame1 = await timeline.getCurrentFrame();
      await timeline.waitForAnimation(1000);
      const frame2 = await timeline.getCurrentFrame();

      expect(frame2).toBeGreaterThan(frame1);
    });
  });

  describe('TC3.2: Zoom In/Out During Playback', () => {
    it('should maintain playback during zoom operations', async () => {
      // Start playback
      await timeline.play();
      
      // Monitor initial performance
      const initialFPS = await timeline.getFPS();
      const initialFrame = await timeline.getCurrentFrame();

      // Zoom in 3 times
      for (let i = 0; i < 3; i++) {
        await page.keyboard.down('Control');
        await page.keyboard.press('+');
        await page.keyboard.up('Control');
        await timeline.waitForAnimation(100);
      }

      // Check performance after zoom in
      let currentFPS = await timeline.getFPS();
      expect(currentFPS).toBeGreaterThanOrEqual(58);

      // Zoom out 5 times
      for (let i = 0; i < 5; i++) {
        await page.keyboard.down('Control');
        await page.keyboard.press('-');
        await page.keyboard.up('Control');
        await timeline.waitForAnimation(100);
      }

      // Check performance after zoom out
      currentFPS = await timeline.getFPS();
      expect(currentFPS).toBeGreaterThanOrEqual(58);

      // Reset zoom
      await page.keyboard.down('Control');
      await page.keyboard.press('0');
      await page.keyboard.up('Control');

      // Verify playback continued throughout
      const finalFrame = await timeline.getCurrentFrame();
      expect(finalFrame).toBeGreaterThan(initialFrame + 50);
    });

    it('should keep playhead centered when possible during zoom', async () => {
      // Seek to middle of timeline
      await timeline.seekToFrame(150);

      // Get initial playhead position
      const getPlayheadScreenPosition = async () => {
        return await page.evaluate(() => {
          const playhead = document.querySelector('[data-testid="playhead"]');
          if (playhead) {
            const rect = playhead.getBoundingClientRect();
            return rect.left + rect.width / 2;
          }
          return 0;
        });
      };

      const initialPosition = await getPlayheadScreenPosition();

      // Zoom in
      await page.keyboard.down('Control');
      await page.keyboard.press('+');
      await page.keyboard.up('Control');

      // Check playhead position after zoom
      const zoomedPosition = await getPlayheadScreenPosition();
      
      // Playhead should remain relatively centered
      expect(Math.abs(zoomedPosition - initialPosition)).toBeLessThan(50);
    });
  });

  describe('TC3.3: Layer Panel Scrolling', () => {
    it('should scroll layer panel independently of timeline', async () => {
      // Add many layers to enable scrolling
      for (let i = 10; i < 30; i++) {
        await timeline.addLayer(`Layer ${i + 1}`);
      }

      // Start playback
      await timeline.play();

      // Scroll layer panel
      const initialLayerScroll = await page.evaluate(() => {
        const layerPanel = document.querySelector('[data-testid="layer-panel"]');
        return layerPanel ? layerPanel.scrollTop : 0;
      });

      await page.evaluate(() => {
        const layerPanel = document.querySelector('[data-testid="layer-panel"]');
        if (layerPanel) {
          layerPanel.scrollTop = 200;
        }
      });

      const newLayerScroll = await page.evaluate(() => {
        const layerPanel = document.querySelector('[data-testid="layer-panel"]');
        return layerPanel ? layerPanel.scrollTop : 0;
      });

      expect(newLayerScroll).toBe(200);
      expect(newLayerScroll).not.toBe(initialLayerScroll);

      // Check playback not affected
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(58);
    });

    it('should handle layer operations during playback', async () => {
      // Start playback
      await timeline.play();

      // Expand/collapse layer folders
      await page.evaluate(() => {
        // Simulate folder toggle
        const folderToggles = document.querySelectorAll('[data-testid^="layer-folder-toggle"]');
        folderToggles.forEach(toggle => {
          (toggle as HTMLElement).click();
        });
      });

      // Select different layers
      await timeline.selectLayer(2);
      await timeline.waitForAnimation(100);
      await timeline.selectLayer(5);
      await timeline.waitForAnimation(100);

      // Toggle layer visibility
      await timeline.toggleLayerVisibility(3);
      await timeline.toggleLayerVisibility(7);

      // Verify performance maintained
      const fps = await timeline.getFPS();
      expect(fps).toBeGreaterThanOrEqual(58);

      // Verify selection highlight updates
      const selectedLayer = await page.evaluate(() => {
        const selected = document.querySelector('[data-testid^="layer-"][data-selected="true"]');
        return selected ? selected.getAttribute('data-testid') : null;
      });

      expect(selectedLayer).toBe('layer-5');
    });
  });
});