<script setup lang="ts">
import { h, type Component } from "vue";

import { ReportData, List, SummaryKpi, Information } from "@vicons/carbon";
import { TargetArrow20Filled } from "@vicons/fluent";

import { NMenu, NIcon, type MenuOption } from "naive-ui";

const selectedKey = defineModel<string>({
  default: "general",
});

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

const menuOptions: MenuOption[] = [
  {
    label: "概览",
    key: "general",
    icon: renderIcon(ReportData),
  },
  {
    label: "伤害",
    key: "damage",
    icon: renderIcon(TargetArrow20Filled),
  },
  {
    label: "任务",
    key: "mission",
    icon: renderIcon(List),
  },
  {
    label: "KPI",
    key: "KPI",
    icon: renderIcon(SummaryKpi),
  },
  {
    label: "信息",
    key: "info",
    icon: renderIcon(Information),
  },
];
</script>
<template>
  <n-menu :options="menuOptions" v-model:value="selectedKey"></n-menu>
</template>
