<script setup lang="ts">
import { NStatistic, NText } from "naive-ui";

const props = defineProps<{
  label: string;
  delta?: { prev: number; total: number };
  reverse?: boolean;
  formatter?: (delta: number) => string;
  style?: string;
  inner_class?: string;
}>();

const deltaRate = (delta: number) => {
  delta *= 100;
  return `${delta > 0 ? "+" + delta.toFixed(1) : delta.toFixed(1)}%`;
};

function getTextType(delta: number) {
  if (delta > 0) {
    if (props.reverse) {
      return "error";
    } else {
      return "success";
    }
  } else if (delta < 0) {
    if (props.reverse) {
      return "success";
    } else {
      return "error";
    }
  } else {
    return "default";
  }
}
</script>
<template>
  <n-statistic :label="label">
    <span :style="style" :class="inner_class">{{ formatter ? formatter(delta?.total || 0) : delta?.total }}</span>
    <template #suffix>
      <n-text
        depth="3"
        :type="
          getTextType(
            props.delta?.total === undefined || props.delta?.prev === undefined
              ? 0
              : props.delta.total - props.delta.prev,
          )
        "
        style="font-size: 0.5rem; vertical-align: top"
        >{{
          deltaRate(
            delta?.total === undefined || delta?.prev === undefined
              ? 0
              : (delta.total - delta.prev) / delta.prev,
          )
        }}</n-text
      >
    </template>
  </n-statistic>
</template>
