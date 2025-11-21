import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./styles/globals.css";
import { initDatabase } from "./services/database";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);

// Initialize database before mounting app
initDatabase()
  .then(() => {
    console.log('✅ Database ready');
    app.mount("#app");
  })
  .catch((error) => {
    console.error('❌ Failed to initialize database:', error);
    // Still mount app even if database fails
    app.mount("#app");
  });
