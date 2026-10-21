import { createApp } from "vue";

(window as any).__main_loaded_time = Date.now();
console.log(
  "[时间戳] main.ts 入口脚本加载执行: %d (延迟: %dms)",
  (window as any).__main_loaded_time,
  (window as any).__main_loaded_time - (window as any).__boot_start_time
);

const application = import.meta.env.DEV && import.meta.env.MODE === "frontend"
  ? import("./dev/HeaderAnimationPreview.vue")
  : import("./App.vue");

void application.then(({ default: Application }) => createApp(Application).mount("#app"))
  .catch((error: unknown) => {
    console.error("ReadUX 启动失败", error);
    const status = document.querySelector(".boot-screen span");
    if (status) status.textContent = `启动失败：${error instanceof Error ? error.message : String(error)}`;
  });
