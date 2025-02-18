<script setup lang="ts">
import {
  useMessage,
  NDataTable,
  NCard,
  NFlex,
  NSelect,
  NModal,
  type DataTableColumns,
} from "naive-ui";
import { ref, h } from "vue";
import { translate } from "@/mapping";
import {formatMissionDate, generateKPICharacterClass, getKPICharacterName } from "@/formatter";
import { kpiCharacterFilterOptions } from "@/tool";

import MissionDetails from "./MissionDetails.vue";
import DeltaView from "./DeltaView.vue";
import type { Response } from "@/type";

interface MissionKPIInfo {
  missionId: number;
  beginTimestamp: number;
  playerIndex: number;
  missionKPI: number;
}

interface PlayerCharacterKPIInfo {
  playerIndex: number;
  characterKPI: number;
  characterKPIType: string;
  missionList: MissionKPIInfo[];
}

type PlayerKPIInfo = Record<
  string,
  { playerIndex: number; playerKPI: number; byCharacter: Record<string, PlayerCharacterKPIInfo> }
>;

interface OverallKPIInfoRow {
  playerName: string;
  playerIndex: number;
  playerKPI: number[];
  children?: OverallKPIInfoRow[];
}

interface PlayerKPIInfoRow {
  missionId: number;
  beginTimestamp: number;
  characterKPIType: string;
  playerIndex: number;
  missionKPI: number;
}

function createOverallKPIInfoTableColumns(): DataTableColumns<OverallKPIInfoRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateKPICharacterClass(row.playerName) },
          getKPICharacterName(row.playerName),
        );
      },
    },
    {
      title: "总玩家指数",
      key: "playerIndex",
      align: "center",
      render(row) {
        return row.playerIndex.toFixed(2);
      },
    },
    {
      title: "KPI",
      key: "playerKPI",
      align: "center",
      defaultSortOrder: "descend",
      sorter: (a, b) => a.playerKPI[1] - b.playerKPI[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.playerKPI[0], total: row.playerKPI[1]},
        formatter: (x) => (x * 100).toFixed(2),
        style: "font-size: 1rem",
      })
    },
  ];
}

function generateOverallKPIInfoTableData(playerKPIData: PlayerKPIInfo) {
  overallKPIInfoTableData.value = [];

  for (const [playerName, playerInfo] of Object.entries(playerKPIData)) {
    let playerMissionList: MissionKPIInfo[] = [];

    for (const characterInfo of Object.values(playerInfo.byCharacter)) {
      for (const missionInfo of characterInfo.missionList) {
        playerMissionList.push(missionInfo);
      }
    }

    playerMissionList.sort((a, b) => a.beginTimestamp - b.beginTimestamp);

    let prevCount = Math.floor(playerMissionList.length * 0.8);
    let prevWeightedSum = 0.0;
    let prevPlayerIndexSum = 0.0;

    for (let i = 0; i < prevCount; i++) {
      prevWeightedSum += playerMissionList[i].missionKPI;
      prevPlayerIndexSum += playerMissionList[i].playerIndex;
    }

    let prevPlayerKPI = prevWeightedSum / prevPlayerIndexSum;

    const row: OverallKPIInfoRow = {
      playerName,
      playerIndex: playerInfo.playerIndex,
      playerKPI: [prevPlayerKPI, playerInfo.playerKPI],
      children: [],
    };

    for (const characterInfo of Object.values(playerInfo.byCharacter)) {
      let characterMissionList = characterInfo.missionList;

      characterMissionList.sort((a, b) => a.beginTimestamp - b.beginTimestamp);
      let prevCount = Math.floor(characterMissionList.length * 0.8);
      let prevWeightedSum = 0.0;
      let prevPlayerIndexSum = 0.0;

      for (let i = 0; i < prevCount; i++) {
        prevWeightedSum += characterMissionList[i].missionKPI;
        prevPlayerIndexSum += characterMissionList[i].playerIndex;
      }

      let prevCharacterKPI = prevWeightedSum / prevPlayerIndexSum;


      row.children?.push({
        playerName: characterInfo.characterKPIType,
        playerIndex: characterInfo.playerIndex,
        playerKPI: [prevCharacterKPI, characterInfo.characterKPI],
      });
    }
    overallKPIInfoTableData.value.push(row);
  }
}

function generatePlayerSelectOptions(playerKPIData: PlayerKPIInfo) {
  for (const playerName of Object.keys(playerKPIData)) {
    playerSelectOptions.value.push({ label: translate(playerName).value, value: playerName });
  }
  playerSelected.value = playerSelectOptions.value[0].value;
}

function createPlayerKPIInfoTableColumns(): DataTableColumns<PlayerKPIInfoRow> {
  return [
    {
      title: "时间",
      key: "beginTimestamp",
      align: "center",
      defaultSortOrder: "descend",
      sortOrder: "descend",
      sorter(a, b) {
        return a.beginTimestamp - b.beginTimestamp;
      },
      render(row) {
        return formatMissionDate(row.beginTimestamp);
      },
    },
    {
      title: "角色",
      key: "characterKPIType",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateKPICharacterClass(row.characterKPIType) },
          getKPICharacterName(row.characterKPIType),
        );
      },
      filterOptions: kpiCharacterFilterOptions,
      filter(value, row) {
        return row.characterKPIType === value;
      },
    },
    {
      title: "玩家指数",
      key: "playerIndex",
      align: "center",
      render(row) {
        return row.playerIndex.toFixed(2);
      },
    },
    {
      title: "任务KPI",
      key: "missionKPI",
      align: "center",
      render(row) {
        return (row.missionKPI * 100).toFixed(2);
      },
      sorter(a, b) {
        return a.missionKPI - b.missionKPI;
      },
    },
  ];
}

function generatePlayerKPIInfoTableData(playerKPIData: PlayerKPIInfo) {
  playerKPIInfoTableData.value = {};
  for (const [playerName, playerInfo] of Object.entries(playerKPIData)) {
    playerKPIInfoTableData.value[playerName] = [];
    for (const characterInfo of Object.values(playerInfo.byCharacter)) {
      for (const missionKPIInfo of characterInfo.missionList) {
        playerKPIInfoTableData.value[playerName].push({
          missionId: missionKPIInfo.missionId,
          beginTimestamp: missionKPIInfo.beginTimestamp,
          characterKPIType: characterInfo.characterKPIType,
          playerIndex: missionKPIInfo.playerIndex,
          missionKPI: missionKPIInfo.missionKPI,
        });
      }
    }
  }
}

const message = useMessage();
const showModal = ref(false);
const showMissionId = ref(1);

const playerKPIData = ref<PlayerKPIInfo>({});
const overallKPIInfoTableData = ref<OverallKPIInfoRow[]>([]);
const playerSelected = ref<string>("");
const playerSelectOptions = ref<{ label: string; value: string }[]>([]);
const playerKPIInfoTableData = ref<Record<string, PlayerKPIInfoRow[]>>({});

const playerKPIInfoTableRowProps = (row: PlayerKPIInfoRow) => {
  return {
    style: "cursor: pointer;",
    onClick: () => {
      showMissionId.value = row.missionId;
      showModal.value = true;
    },
  };
};

fetch("./api/kpi/player_kpi")
  .then((response) => response.json())
  .then((data: Response<PlayerKPIInfo>) => {
    if (data.code !== 200) {
      message.error(`API error while fetching player KPI data: ${data.code} ${data.message}`);
    } else {
      playerKPIData.value = data.data;
      generateOverallKPIInfoTableData(playerKPIData.value);
      generatePlayerSelectOptions(playerKPIData.value);
      generatePlayerKPIInfoTableData(playerKPIData.value);
    }
  })
  .catch((error) => {
    message.error("HTTP error while fetching player KPI data: " + error);
  });
</script>

<template>
  <n-card title="总览">
    <n-flex justify="center">
      <n-data-table
        :columns="createOverallKPIInfoTableColumns()"
        :data="overallKPIInfoTableData"
        :pagination="false"
        :row-key="(row: OverallKPIInfoRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
    </n-flex>
  </n-card>
  <n-card title="按玩家">
    <n-select
      v-model:value="playerSelected"
      :options="playerSelectOptions"
      style="max-width: 250px"
    ></n-select>
    <n-flex justify="center">
      <n-data-table
        :columns="createPlayerKPIInfoTableColumns()"
        :data="playerKPIInfoTableData[playerSelected]"
        :pagination="{ pageSize: 10 }"
        :row-props="playerKPIInfoTableRowProps"
        :row-key="(row: PlayerKPIInfoRow) => row.missionId"
        style="width: fit-content"
      ></n-data-table>
    </n-flex>
  </n-card>
  <n-modal v-model:show="showModal">
    <n-card
      style="width: 80%"
      display-directive="show"
      title="任务信息"
      :bordered="false"
      size="huge"
      role="card"
      aria-modal="true"
      closable
      :on-close="() => (showModal = false)"
    >
      <MissionDetails :mission-id="showMissionId" style="width: 600px"></MissionDetails>
    </n-card>
  </n-modal>
</template>
