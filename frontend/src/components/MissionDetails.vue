<script setup lang="ts">
import {
  NButton,
  NIcon,
  NCard,
  NTabs,
  NH2,
  NTabPane,
  useMessage,
  NMessageProvider,
} from "naive-ui";
import { formatMissionDate } from "@/formatter";
import { watch, ref } from "vue";
import { ArrowLeft } from "@vicons/carbon";
import { useRoute, useRouter } from "vue-router";

import type { Response } from "@/type";

import MissionGeneral from "./MissionGeneral.vue";
import MissionDamage from "./MissionDamage.vue";
import MissionPick from "./MissionPick.vue";
import MissionKPI from "./MissionKPI.vue";

const route = useRoute();
const router = useRouter();
const message = useMessage();

const props = defineProps<{
  missionId?: number;
}>();

interface MissionGeneralInfo {
  missionId: number;
  missionBeginTimestamp: number;
  missionInvalid: boolean;
  missionInvalidReason: string;
}

const missionGeneralData = ref<MissionGeneralInfo>({
  missionId: 1,
  missionBeginTimestamp: 0,
  missionInvalid: false,
  missionInvalidReason: "",
});

watch(
  () => route.params.id,
  () => {
    let missionId: number;
    if (props.missionId === undefined) {
      missionId = Number(route.params.id);
    } else {
      missionId = props.missionId;
    }

    if (isNaN(missionId)) {
      router.push({ name: "notFound" });
    } else {
      missionGeneralData.value.missionId = missionId;
      fetch(`./api/mission/${missionGeneralData.value.missionId}/info`)
        .then((response) => response.json())
        .then((data: Response<MissionGeneralInfo>) => {
          if (data.code === 404) {
            router.push({ name: "notFound" });
          } else if (data.code !== 200) {
            message.error(`API error while fetching mission data: ${data.code} ${data.message}`);
          } else {
            missionGeneralData.value = data.data;
            missionGeneralData.value.missionId = missionId;
          }
        })
        .catch((error) => {
          message.error("HTTP error while fetching mission data: " + error);
        });
    }
  },
  { immediate: true },
);
</script>

<template>
  <n-h2 style="padding: 10px; margin: 0px">
    <n-button
      v-if="!props.missionId"
      text
      style="font-size: 24px"
      @click="$router.push({ name: 'mission' })"
    >
      <template #icon>
        <n-icon><ArrowLeft /></n-icon>
      </template>
    </n-button>
    {{ formatMissionDate(missionGeneralData.missionBeginTimestamp) }}
  </n-h2>
  <n-card>
    <n-tabs type="line" default-value="general">
      <n-tab-pane name="general" tab="概览" display-directive="show:lazy">
        <n-message-provider>
          <MissionGeneral
            :mission-id="missionGeneralData.missionId"
            :mission-invalid="missionGeneralData.missionInvalid"
            :mission-invalid-reason="missionGeneralData.missionInvalidReason"
          />
        </n-message-provider>
      </n-tab-pane>
      <n-tab-pane name="damage" tab="伤害" display-directive="show:lazy">
        <n-message-provider>
          <MissionDamage :mission-id="missionGeneralData.missionId" />
        </n-message-provider>
      </n-tab-pane>
      <n-tab-pane name="pick" tab="采集" display-directive="show:lazy">
        <n-message-provider>
          <MissionPick :mission-id="missionGeneralData.missionId" />
        </n-message-provider>
      </n-tab-pane>
      <n-tab-pane name="kpi" tab="KPI" display-directive="show:lazy">
        <n-message-provider>
          <MissionKPI :mission-id="missionGeneralData.missionId" />
        </n-message-provider>
      </n-tab-pane>
    </n-tabs>
  </n-card>
</template>
