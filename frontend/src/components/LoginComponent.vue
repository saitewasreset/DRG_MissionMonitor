<script setup lang="ts">
import { ref } from "vue";
import { NCard, NForm, NFormItem, NInput, NButton, useMessage } from "naive-ui";
import { useLoginStore } from "@/store/login";

interface LoginResponse {
  code: number;
  message: string;
}

const accessToken = ref("");
const message = useMessage();
const loginStatus = useLoginStore();
const loginButtonDisabled = ref(false);

function login() {
  loginButtonDisabled.value = true;
  fetch(`./api/login`, {
    method: "POST",
    body: accessToken.value,
  })
    .then((res) => res.json())
    .then((data: LoginResponse) => {
      if (data.code == 200) {
        message.info(data.message);
        loginStatus.login = true;
      } else if (data.code == 403) {
        message.error(data.message);
        accessToken.value = "";
      } else {
        message.error(`API Error: ${data.code} ${data.message}`);
      }
    })
    .catch((e) => {
      message.error(`HTTP Error: ${e}`);
    })
    .finally(() => {
      loginButtonDisabled.value = false;
    });
}
</script>
<template>
  <n-card title="需要认证" style="width: 80%">
    <n-form label-placement="left" label-width="auto">
      <n-form-item label="Access Token">
        <n-input v-model:value="accessToken" type="password"></n-input>
      </n-form-item>

      <div style="display: flex; justify-content: flex-end">
        <n-button attr-type="button" @click="login" :disabled="loginButtonDisabled">
          认证
        </n-button>
      </div>
    </n-form>
  </n-card>
</template>
