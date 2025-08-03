import { Page, ElementHandle } from 'puppeteer';

export class TimelineHelpers {
  constructor(private page: Page) {}

  // Selectors
  private selectors = {
    timeline: '[data-testid="timeline"]',
    playButton: '[data-testid="play-button"]',
    pauseButton: '[data-testid="pause-button"]',
    stopButton: '[data-testid="stop-button"]',
    playhead: '[data-testid="playhead"]',
    frameCounter: '[data-testid="frame-counter"]',
    fpsDisplay: '[data-testid="fps-display"]',
    ruler: '[data-testid="timeline-ruler"]',
    frameGrid: '[data-testid="frame-grid"]',
    layerPanel: '[data-testid="layer-panel"]',
    layer: '[data-testid^="layer-"]',
    addLayerButton: '[data-testid="add-layer-button"]',
    stage: '[data-testid="stage"]',
    stageObject: '[data-testid^="stage-object-"]',
    scriptEditor: '[data-testid="script-editor"]',
    console: '[data-testid="console"]',
    consoleLog: '[data-testid^="console-log-"]',
    libraryPanel: '[data-testid="library-panel"]',
    propertiesPanel: '[data-testid="properties-panel"]',
    sceneTab: '[data-testid^="scene-tab-"]',
    zoomSlider: '[data-testid="zoom-slider"]',
  };

  // Navigation
  async waitForTimeline(): Promise<void> {
    await this.page.waitForSelector(this.selectors.timeline, { timeout: 10000 });
  }

  async play(): Promise<void> {
    await this.page.click(this.selectors.playButton);
  }

  async pause(): Promise<void> {
    await this.page.click(this.selectors.pauseButton);
  }

  async stop(): Promise<void> {
    await this.page.click(this.selectors.stopButton);
  }

  async getCurrentFrame(): Promise<number> {
    const frameText = await this.page.$eval(this.selectors.frameCounter, el => el.textContent);
    return parseInt(frameText || '0');
  }

  async getFPS(): Promise<number> {
    const fpsText = await this.page.$eval(this.selectors.fpsDisplay, el => el.textContent);
    const match = fpsText?.match(/(\d+(?:\.\d+)?)/);
    return match ? parseFloat(match[1]) : 0;
  }

  async seekToFrame(frame: number): Promise<void> {
    const ruler = await this.page.$(this.selectors.ruler);
    if (!ruler) throw new Error('Timeline ruler not found');

    const box = await ruler.boundingBox();
    if (!box) throw new Error('Could not get ruler bounds');

    // Calculate click position based on frame
    const frameWidth = box.width / 300; // Assuming 300 total frames
    const x = box.x + (frame * frameWidth);
    const y = box.y + (box.height / 2);

    await this.page.mouse.click(x, y);
  }

  async dragPlayhead(startFrame: number, endFrame: number): Promise<void> {
    const playhead = await this.page.$(this.selectors.playhead);
    if (!playhead) throw new Error('Playhead not found');

    const ruler = await this.page.$(this.selectors.ruler);
    if (!ruler) throw new Error('Timeline ruler not found');

    const box = await ruler.boundingBox();
    if (!box) throw new Error('Could not get ruler bounds');

    const frameWidth = box.width / 300;
    const startX = box.x + (startFrame * frameWidth);
    const endX = box.x + (endFrame * frameWidth);
    const y = box.y + (box.height / 2);

    await this.page.mouse.move(startX, y);
    await this.page.mouse.down();
    await this.page.mouse.move(endX, y, { steps: 10 });
    await this.page.mouse.up();
  }

  // Layer operations
  async getLayers(): Promise<ElementHandle[]> {
    return await this.page.$$(this.selectors.layer);
  }

  async addLayer(name?: string): Promise<void> {
    await this.page.click(this.selectors.addLayerButton);
    
    if (name) {
      // Wait for the text input to appear and type the name
      await this.page.waitForSelector('input[type="text"]:focus', { timeout: 2000 });
      await this.page.keyboard.type(name);
      await this.page.keyboard.press('Enter');
    }
  }

  async selectLayer(index: number): Promise<void> {
    const layers = await this.getLayers();
    if (index < layers.length) {
      await layers[index].click();
    }
  }

  async toggleLayerVisibility(index: number): Promise<void> {
    const layers = await this.getLayers();
    if (index < layers.length) {
      const visibilityToggle = await layers[index].$('[data-testid="layer-visibility"]');
      if (visibilityToggle) {
        await visibilityToggle.click();
      }
    }
  }

  // Stage operations
  async getStageObjects(): Promise<ElementHandle[]> {
    return await this.page.$$(this.selectors.stageObject);
  }

  async selectStageObject(id: string): Promise<void> {
    await this.page.click(`[data-testid="stage-object-${id}"]`);
  }

  async dragStageObject(id: string, deltaX: number, deltaY: number): Promise<void> {
    const object = await this.page.$(`[data-testid="stage-object-${id}"]`);
    if (!object) throw new Error(`Stage object ${id} not found`);

    const box = await object.boundingBox();
    if (!box) throw new Error('Could not get object bounds');

    const startX = box.x + box.width / 2;
    const startY = box.y + box.height / 2;

    await this.page.mouse.move(startX, startY);
    await this.page.mouse.down();
    await this.page.mouse.move(startX + deltaX, startY + deltaY, { steps: 10 });
    await this.page.mouse.up();
  }

  // Script operations
  async openScriptEditor(): Promise<void> {
    await this.page.keyboard.press('F9');
    await this.page.waitForSelector(this.selectors.scriptEditor, { timeout: 2000 });
  }

  async setFrameScript(frame: number, script: string): Promise<void> {
    await this.seekToFrame(frame);
    await this.openScriptEditor();
    
    // Clear existing script
    await this.page.keyboard.down('Control');
    await this.page.keyboard.press('a');
    await this.page.keyboard.up('Control');
    
    // Type new script
    await this.page.keyboard.type(script);
    
    // Save script (Ctrl+S)
    await this.page.keyboard.down('Control');
    await this.page.keyboard.press('s');
    await this.page.keyboard.up('Control');
  }

  async getConsoleOutput(): Promise<string[]> {
    const logs = await this.page.$$(this.selectors.consoleLog);
    const output: string[] = [];
    
    for (const log of logs) {
      const text = await log.evaluate(el => el.textContent);
      if (text) output.push(text);
    }
    
    return output;
  }

  // Performance monitoring
  async measureFPS(duration: number): Promise<{ avg: number, min: number, max: number }> {
    const samples: number[] = [];
    const startTime = Date.now();
    
    while (Date.now() - startTime < duration) {
      const fps = await this.getFPS();
      samples.push(fps);
      await this.page.waitForTimeout(100);
    }
    
    return {
      avg: samples.reduce((a, b) => a + b, 0) / samples.length,
      min: Math.min(...samples),
      max: Math.max(...samples),
    };
  }

  async measureSeekTime(targetFrame: number): Promise<number> {
    const startTime = Date.now();
    await this.seekToFrame(targetFrame);
    
    // Wait for frame to update
    await this.page.waitForFunction(
      (frame) => {
        const counter = document.querySelector('[data-testid="frame-counter"]');
        return counter && parseInt(counter.textContent || '0') === frame;
      },
      { timeout: 5000 },
      targetFrame
    );
    
    return Date.now() - startTime;
  }

  // Utility functions
  async waitForFrame(frame: number, timeout: number = 5000): Promise<void> {
    await this.page.waitForFunction(
      (targetFrame) => {
        const counter = document.querySelector('[data-testid="frame-counter"]');
        return counter && parseInt(counter.textContent || '0') === targetFrame;
      },
      { timeout },
      frame
    );
  }

  async waitForAnimation(duration: number): Promise<void> {
    await this.page.waitForTimeout(duration);
  }

  async takeScreenshot(name: string): Promise<void> {
    await this.page.screenshot({ 
      path: `screenshots/${name}.png`,
      fullPage: true 
    });
  }

  async getMemoryUsage(): Promise<{ heapUsed: number, external: number }> {
    return await this.page.evaluate(() => {
      if (performance && 'memory' in performance) {
        return {
          heapUsed: (performance as any).memory.usedJSHeapSize,
          external: (performance as any).memory.totalJSHeapSize
        };
      }
      return { heapUsed: 0, external: 0 };
    });
  }
}