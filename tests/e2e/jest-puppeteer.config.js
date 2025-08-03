module.exports = {
  launch: {
    headless: process.env.HEADLESS === 'true',
    slowMo: process.env.DEBUG === 'true' ? 100 : 0,
    devtools: process.env.DEBUG === 'true',
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--disable-web-security',
      '--disable-features=IsolateOrigins,site-per-process'
    ],
    defaultViewport: {
      width: 1920,
      height: 1080
    }
  },
  server: {
    command: 'cd ../../nannou_timeline/standalone_demo && cargo run --bin timeline-demo-web --release',
    port: 8080,
    launchTimeout: 60000,
    debug: true
  }
};