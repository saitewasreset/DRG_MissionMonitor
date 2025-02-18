<script setup lang="ts">
import { ref, watch, computed, onMounted } from "vue";
import { NConfigProvider, zhCN, dateZhCN, darkTheme, lightTheme, useOsTheme } from "naive-ui";
import {
  NAlert,
  NIcon,
  NLayout,
  NLayoutHeader,
  NLayoutSider,
  NLayoutContent,
  NLayoutFooter,
  NTooltip,
  NSpace,
  NFlex,
  NButton,
  NGlobalStyle,
  NMessageProvider,
} from "naive-ui";
import { mappingError } from "./mapping";
import { TaskAssetView, FavoriteFilled, LogoGithub, Sun, Moon, Cloudy } from "@vicons/carbon";
import NavBar from "./components/NavBar.vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import LoginStatusComponent from "@/components/LoginStatusComponent.vue";
import { useLoginStore } from "@/store/login";

const route = useRoute();
const router = useRouter();
const currentSelectComponentKey = ref("general");
const loginStore = useLoginStore();

loginStore.checkLogin();

watch(
  () => route.name,
  () => {
    if (route.name != null) {
      if (typeof route.name === "string") {
        currentSelectComponentKey.value = route.name;
      }
    }
  },
);

watch(currentSelectComponentKey, (value) => {
  if (value != null) {
    router.push({ name: value });
  }
});

// get os theme
const osTheme = useOsTheme();
const userTheme = ref<null | "light" | "dark">(null);
const followSystemTheme = ref(true);

onMounted(() => {
  const savedTheme = localStorage.getItem("theme");
  if (savedTheme === "light" || savedTheme === "dark") {
    userTheme.value = savedTheme;
    followSystemTheme.value = false;
  } else {
    followSystemTheme.value = true;
  }
});

const theme = computed(() => {
  if (followSystemTheme.value) {
    return osTheme.value === "dark" ? darkTheme : lightTheme;
  }
  return userTheme.value === "dark" ? darkTheme : lightTheme;
});

const setTheme = (theme: "light" | "dark" | "system") => {
  if (theme === "system") {
    followSystemTheme.value = true;
  } else {
    followSystemTheme.value = false;
    userTheme.value = theme;
  }
  localStorage.setItem("theme", theme);
};

const setLightTheme = () => setTheme("light");
const setDarkTheme = () => setTheme("dark");
const setSystemTheme = () => setTheme("system");
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme="theme">
    <n-message-provider>
      <n-global-style />
      <n-space vertical size="large">
        <n-layout position="absolute" style="min-width: 1400px">
          <n-layout-header bordered style="height: 64px">
            <div class="title">
              <div>
                <n-icon size="1.2rem">
                  <TaskAssetView />
                </n-icon>
                Mission Monitor
              </div>
            </div>
            <div>
              <n-alert class="alert-box" v-if="mappingError != null" type="error">
                无法加载Mapping：{{ mappingError }}
              </n-alert>
            </div>
            <div></div>
            <div></div>
            <div class="theme-switch">
              <LoginStatusComponent></LoginStatusComponent>
              <div class="theme-button-wrapper">
                <n-tooltip trigger="hover" placement="bottom">
                  <template #trigger>
                    <n-button
                      text
                      @click="setLightTheme"
                      :disabled="!followSystemTheme && userTheme === 'light'"
                    >
                      <n-icon size="1.25rem">
                        <Sun />
                      </n-icon>
                    </n-button>
                  </template>
                  <span>浅色模式</span>
                </n-tooltip>
                <n-tooltip trigger="hover" placement="bottom">
                  <template #trigger>
                    <n-button text @click="setSystemTheme" :disabled="followSystemTheme">
                      <n-icon size="1.25rem">
                        <Cloudy />
                      </n-icon>
                    </n-button>
                  </template>
                  <span>跟随系统</span>
                </n-tooltip>
                <n-tooltip trigger="hover" placement="bottom">
                  <template #trigger>
                    <n-button
                      text
                      @click="setDarkTheme"
                      :disabled="!followSystemTheme && userTheme === 'dark'"
                    >
                      <n-icon size="1.25rem">
                        <Moon />
                      </n-icon>
                    </n-button>
                  </template>
                  <span>深色模式</span>
                </n-tooltip>
              </div>
            </div>
          </n-layout-header>
          <n-layout has-sider position="absolute" style="top: 64px; bottom: 64px">
            <n-layout-sider
              bordered
              collapse-mode="width"
              :width="200"
              :collapsed-width="50"
              show-trigger="arrow-circle"
            >
              <NavBar v-model="currentSelectComponentKey"></NavBar>
            </n-layout-sider>
            <n-layout-content>
              <RouterView></RouterView>
            </n-layout-content>
          </n-layout>
          <n-layout-footer bordered position="absolute" style="height: 64px; padding: 10px">
            <n-flex align="center" vertical>
              <p class="footer-p">同舟共济，绝不放弃！</p>
              <p class="footer-p">
                made by
                <a href="https://github.com/saitewasreset" target="_blank" class="author admin"
                  >saitewasreset</a
                >
                &
                <a href="https://github.com/DeepChirp" target="_blank" class="author gunner"
                  >深鸣</a
                >
                with
                <n-icon color="red" size="8">
                  <FavoriteFilled />
                </n-icon>
                &ensp;
                <n-button
                  text
                  tag="a"
                  target="_blank"
                  href="https://github.com/saitewasreset/DRG_MissionMonitor-webui"
                  ><n-icon size="10"> <LogoGithub /> </n-icon>Source</n-button
                >
              </p>
            </n-flex>
          </n-layout-footer>
        </n-layout>
      </n-space>
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
.n-layout-header {
  display: grid;
  grid-template-columns: 2fr 5fr 5fr 5fr 1.2fr;
  max-height: 10%;
}

.title {
  font-size: 1.2rem;
  display: flex;
  align-items: center;
  text-align: center;
  max-width: 200px;
}

.footer-p {
  text-align: center;
  margin: auto;
  font-size: 0.8rem;
}

.theme-switch {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 16px;
  padding-right: 16px;
}

.theme-button-wrapper {
  display: flex;
  gap: 8px;
}

.n-button.active {
  background-color: var(--n-color-primary);
  color: white;
  border-radius: 50%;
}

a.author {
  text-decoration: none;
}
</style>
