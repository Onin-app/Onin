// Hello World Plugin
// This is a simple example plugin for the Baize plugin system

class HelloWorldPlugin {
    constructor() {
        this.name = 'Hello World Plugin';
        this.isActive = false;
    }

    async activate(context) {
        console.log('Hello World Plugin activated!');
        this.context = context;
        this.isActive = true;
        
        // Show a notification when activated
        if (context.app && context.app.showNotification) {
            await context.app.showNotification('Hello World Plugin activated!');
        }
        
        // Listen for events
        if (context.events) {
            context.events.on('app:ready', this.onAppReady.bind(this));
        }
    }

    async deactivate() {
        console.log('Hello World Plugin deactivated!');
        this.isActive = false;
        
        // Clean up event listeners
        if (this.context && this.context.events) {
            this.context.events.off('app:ready', this.onAppReady.bind(this));
        }
        
        this.context = null;
    }

    onAppReady() {
        console.log('Hello World Plugin: App is ready!');
    }
}

// Export the plugin class
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HelloWorldPlugin;
} else if (typeof window !== 'undefined') {
    window.HelloWorldPlugin = HelloWorldPlugin;
}