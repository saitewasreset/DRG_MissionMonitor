<script setup lang="ts">
import { ref, watch } from "vue";
import { NButton, NIcon, NModal } from "naive-ui";
import { Locked, Unlocked } from "@vicons/carbon";

import { useLoginStore } from "@/store/login";
import LoginComponent from "@/components/LoginComponent.vue";

const loginStore = useLoginStore();
const showLogin = ref(false);

interface LogoutResponse {
  code: number;
  message: string;
}

function onLockClicked() {
  if (loginStore.login === false) {
    showLogin.value = true;
  } else {
    fetch("./api/logout", {
      method: "POST",
    })
      .then((res) => res.json())
      .then((data: LogoutResponse) => {
        if (data.code == 200) {
          loginStore.login = false;
        }
      });
  }
}

watch(
  () => loginStore.login,
  () => {
    if (loginStore.login) {
      showLogin.value = false;
    }
  },
);
</script>
<template>
  <n-button text @click="onLockClicked">
    <n-icon size="1.25rem">
      <Unlocked v-if="loginStore.login"></Unlocked>
      <Locked v-else></Locked>
    </n-icon>
  </n-button>
  <n-modal v-model:show="showLogin">
    <LoginComponent></LoginComponent>
  </n-modal>
</template>
