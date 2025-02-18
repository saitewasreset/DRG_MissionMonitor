import "./assets/base.css";
import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "vfonts/Roboto.css";
import "vfonts/FiraCode.css";
import { createWebHashHistory, createRouter } from "vue-router";

const MissionComponent = () => import("./components/MissionComponent.vue");
const MissionList = () => import("./components/MissionList.vue");
const MissionDetails = () => import("./components/MissionDetails.vue");
const GeneralComponent = () => import("./components/GeneralComponent.vue");
const DamageComponent = () => import("./components/DamageComponent.vue");
const KPIComponent = () => import("./components/KPIComponent.vue");
const InfoComponent = () => import("./components/InfoComponent.vue");
const NotFound = () => import("./components/NotFound.vue");

const routes = [
  { name: "general", path: "/", component: GeneralComponent },
  { name: "damage", path: "/damage", component: DamageComponent },
  {
    path: "/mission",
    component: MissionComponent,
    children: [
      {
        name: "mission",
        path: "",
        component: MissionList,
      },
      {
        name: "missionDetails",
        path: ":id",
        component: MissionDetails,
      },
    ],
  },
  { name: "KPI", path: "/kpi", component: KPIComponent },
  { name: "info", path: "/info", component: InfoComponent },
  { name: "notFound", path: "/:pathMatch(.*)", component: NotFound },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const pinia = createPinia();

createApp(App).use(pinia).use(router).mount("#app");
