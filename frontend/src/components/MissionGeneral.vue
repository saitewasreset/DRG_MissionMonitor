<script setup lang="ts">
interface MissionGeneralPlayerInfo {
  characterGameId: string;
  playerRank: number;
  characterRank: number;
  characterPromotion: number;
  presentTime: number;
  reviveNum: number;
  deathNum: number;
  playerEscaped: number;
}

interface MissionGeneralTableRowData {
  playerName: string;
  heroGameId: string;
  playerRank: number;
  characterRank: number;
  characterTitle: string;
  characterPromotion: number;
  playerIndex: string;
  reviveNum: number;
  deathNum: number;
  playerEscaped: string;
}

interface MissionGeneralData {
  beginTimeStamp: number;
  hazardId: number;
  missionResult: number;
  missionTime: number;
  missionTypeId: string;
  playerInfo: Record<string, MissionGeneralPlayerInfo>;
  rewardCredit: number;
  totalDamage: number;
  totalKill: number;
  totalMinerals: number;
  totalNitra: number;
  totalSupplyCount: number;
}

import { ref, computed, h, watch } from "vue";

import {
  useMessage,
  NGrid,
  NGi,
  NStatistic,
  NDataTable,
  NCard,
  type DataTableColumns,
} from "naive-ui";

import {
  formatMissionTime,
  formatMissionResult,
  formatMissionDifficulty,
  formatCharacterTitle,
  nFormatter,
  generateCharacterClass,
  generateTitleClass,
} from "../formatter";

import { translate } from "../mapping";
import type { Response } from "@/type";

const missionData = ref<MissionGeneralData | null>(null);

const tableData = ref<MissionGeneralTableRowData[]>([]);

const tableColumn = ref<DataTableColumns<MissionGeneralTableRowData>>(createColumns());

const message = useMessage();

const props = defineProps<{
  missionId?: number;
  missionInvalid?: boolean;
  missionInvalidReason?: string;
}>();

function generateTableData(
  playerInfo: Record<string, MissionGeneralPlayerInfo>,
  missionTime: number,
): MissionGeneralTableRowData[] {
  let result = [];
  for (const [playerName, playerData] of Object.entries(playerInfo)) {
    result.push({
      playerName,
      heroGameId: playerData.characterGameId,
      playerRank: playerData.playerRank,
      characterRank: playerData.characterRank,
      characterTitle: formatCharacterTitle(playerData.characterPromotion),
      characterPromotion: playerData.characterPromotion,
      playerIndex: (playerData.presentTime / missionTime).toFixed(2),
      reviveNum: playerData.reviveNum,
      deathNum: playerData.deathNum,
      playerEscaped: playerData.playerEscaped == 1 ? "是" : "否",
    });
  }

  return result;
}

function createColumns(): DataTableColumns<MissionGeneralTableRowData> {
  return [
    {
      title: "玩家",
      key: "playerName",
      align: "center",
      render(row) {
        return h("span", { class: `${generateCharacterClass(row.heroGameId)}` }, row.playerName);
      },
    },
    {
      title: "蓝等",
      key: "playerRank",
      align: "center",
    },
    {
      title: "红等",
      key: "characterRank",
      align: "center",
    },
    {
      title: "头衔",
      key: "characterTitle",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: `${generateTitleClass(row.characterPromotion)}` },
          row.characterTitle,
        );
      },
    },
    {
      title: "玩家指数",
      key: "playerIndex",
      align: "center",
    },
    {
      title: "救人次数",
      key: "reviveNum",
      align: "center",
    },
    {
      title: "倒地次数",
      key: "deathNum",
      align: "center",
    },
    {
      title: "成功撤离",
      key: "playerEscaped",
      align: "center",
    },
  ];
}

watch(
  () => props.missionId,
  () => {
    fetch(`./api/mission/${props.missionId == undefined ? 1 : props.missionId}/general`)
      .then((data) => data.json())
      .then((json: Response<MissionGeneralData>) => {
        if (json.code !== 200) {
          message.error(`API Error: ${json.code} ${json.message}`);
        } else {
          missionData.value = json.data;
          tableData.value = generateTableData(json.data.playerInfo, json.data.missionTime);
        }
      })
      .catch((e) => {
        message.error(`HTTP Error: ${e}`);
      });
  },
  { immediate: true },
);
</script>

<template>
  <main>
    <n-card title="任务概览">
      <n-grid :cols="4">
        <n-gi>
          <n-statistic label="任务时长">{{
            computed(() => formatMissionTime(missionData?.missionTime))
          }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="难度">{{
            computed(() => formatMissionDifficulty(missionData?.hazardId))
          }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="任务状态">{{
            computed(() => formatMissionResult(missionData?.missionResult))
          }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="任务类型"> {{ translate(missionData?.missionTypeId) }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="总击杀数">{{ missionData?.totalKill }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="总伤害">{{
            computed(() => nFormatter(missionData?.totalDamage, 2))
          }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="采集硝石量">{{
            computed(() => nFormatter(missionData?.totalNitra, 2))
          }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="采集矿石量">{{
            computed(() => nFormatter(missionData?.totalMinerals, 2))
          }}</n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="奖励代币">
            {{ computed(() => nFormatter(missionData?.rewardCredit, 2)) }}
          </n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="使用补给份数">
            {{ missionData?.totalSupplyCount }}
          </n-statistic>
        </n-gi>
        <n-gi>
          <n-statistic label="有效性">
            {{ missionInvalid === true ? `无效：${missionInvalidReason}` : "有效" }}
          </n-statistic>
        </n-gi>
      </n-grid>
    </n-card>
    <n-card title="玩家">
      <div class="data-table-container">
        <n-data-table
          :columns="tableColumn"
          :data="tableData"
          :pagination="false"
          :bordered="true"
          :row-key="(row: MissionGeneralTableRowData) => row.heroGameId"
          style="width: fit-content"
        ></n-data-table>
      </div>
    </n-card>
  </main>
</template>

<style>
.header-time-relative {
  font-size: 0.5rem;
}

.data-table-container {
  display: flex;
  justify-content: space-around;
}
</style>
