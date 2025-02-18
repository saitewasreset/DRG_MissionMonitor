<script setup lang="ts">
import { NMessageProvider, NTabs, NTabPane, NCard } from "naive-ui";
import KPIFactor from "./KPIFactor.vue";
import KPIPlayer from "./KPIPlayer.vue";
</script>
<template>
  <NMessageProvider>
    <n-card>
      <n-tabs type="line" default-value="info">
        <n-tab-pane name="info" tab="信息" display-directive="show:lazy">
          <KPIFactor></KPIFactor>
        </n-tab-pane>
        <n-tab-pane name="player" tab="玩家KPI" display-directive="show:lazy">
          <KPIPlayer></KPIPlayer>
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </NMessageProvider>
</template>
