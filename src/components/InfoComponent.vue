<script setup lang="ts">
import { NCard, NTabs, NTabPane, NMessageProvider } from "naive-ui";
import InfoBrothers from "./InfoBrothers.vue";
import InfoWeaponPreference from "./InfoWeaponPreference.vue";
</script>
<template>
  <NMessageProvider>
    <n-card>
      <n-tabs type="line" default-value="brothers">
        <n-tab-pane name="brothers" tab="素不相识的矮人们" display-directive="show:lazy">
          <InfoBrothers></InfoBrothers>
        </n-tab-pane>
        <n-tab-pane name="weapon_preference" tab="武器偏好" display-directive="show:lazy">
          <InfoWeaponPreference></InfoWeaponPreference>
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </NMessageProvider>
</template>
