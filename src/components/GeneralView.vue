<script setup lang="ts">
import {
  NCard,
  NDataTable,
  NGrid,
  NGridItem,
  NStatistic,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import { ref, h, watch } from "vue";

import DeltaView from "./DeltaView.vue";

import { formatMissionTime, formatPercent, nFormatter, generateCharacterClass } from "@/formatter";

import Plotly, { type Data } from "plotly.js-basic-dist";

import type { Response } from "@/type";

interface DeltaData {
  prev: number;
  recent: number;
  total: number;
}

interface GeneralInfo {
  gameCount: number;
  validRate: number;
  totalMissionTime: number;
  averageMissionTime: DeltaData;
  uniquePlayerCount: number;
  openRoomRate: DeltaData;
  passRate: DeltaData;
  averageDifficulty: DeltaData;
  averageKillNum: DeltaData;
  averageDamage: DeltaData;
  averageDeathNumPerPlayer: DeltaData;
  averageMineralsMined: DeltaData;
  averageSupplyCountPerPlayer: DeltaData;
  averageRewardCredit: DeltaData;
}

interface MissionTypeInfo {
  averageDifficulty: number;
  averageMissionTime: number;
  averageRewardCredit: number;
  creditPerMinute: number;
  missionCount: number;
  passRate: number;
}

interface MissionInfo {
  missionTypeMap: Record<string, string>;
  missionTypeData: Record<string, MissionTypeInfo>;
}

interface PlayerInfo {
  averageDeathNum: number;
  averageMineralsMined: number;
  averageReviveNum: number;
  averageSupplyCount: number;
  averageSupplyEfficiency: number;
  characterInfo: Record<string, number>;
  validMissionCount: number;
}

interface PlayerData {
  characterMap: Record<string, string>;
  playerData: Record<string, PlayerInfo>;
  prevPlayerData: Record<string, PlayerInfo>;
}

interface CharacterInfo {
  characterChoiceCount: Record<string, number>;
  characterMapping: Record<string, string>;
}

interface CharacterGeneralData {
  playerIndex: number;
  reviveNum: number;
  deathNum: number;
  mineralsMined: number;
  supplyCount: number;
  supplyEfficiency: number;
}

interface CharacterGeneralInfo {
  characterData: Record<string, CharacterGeneralData>;
  characterMapping: Record<string, string>;
}

interface MissionTypeInfoTableRow extends MissionTypeInfo {
  missionType: string;
  mappedType: string;
}

interface PlayerInfoTableRow {
  playerName: string;
  averageDeathNum: number[];
  averageMineralsMined: number[];
  averageReviveNum: number[];
  averageSupplyCount: number[];
  averageSupplyEfficiency: number[];
  characterInfo: Record<string, number>;
  validMissionCount: number;
  characterMapping: Record<string, string>;
}

interface CharacterInfoTableRow extends CharacterGeneralData {
  characterGameId: string;
  characterName: string;
}

function createMissionTypeTableColumns(): DataTableColumns<MissionTypeInfoTableRow> {
  return [
    {
      title: "任务类型",
      key: "missionType",
      render: (row) => row.mappedType,
      align: "center",
    },
    {
      title: "有效局数",
      key: "missionCount",
      sorter: (a, b) => a.missionCount - b.missionCount,
      align: "center",
    },
    {
      title: "通过率",
      key: "passRate",
      render: (row) => formatPercent(row.passRate),
      sorter: (a, b) => a.passRate - b.passRate,
      align: "center",
    },
    {
      title: "平均难度",
      key: "averageDifficulty",
      render: (row) => row.averageDifficulty.toFixed(2),
      sorter: (a, b) => a.averageDifficulty - b.averageDifficulty,
      align: "center",
    },
    {
      title: "平均任务时长",
      key: "averageMissionTime",
      render: (row) => formatMissionTime(row.averageMissionTime),
      sorter: (a, b) => a.averageMissionTime - b.averageMissionTime,
      align: "center",
    },
    {
      title: "平均奖励代币",
      key: "averageRewardCredit",
      render: (row) => nFormatter(row.averageRewardCredit, 2),
      sorter: (a, b) => a.averageRewardCredit - b.averageRewardCredit,
      align: "center",
    },
    {
      title: "代币效率",
      key: "creditPerMinute",
      render: (row) => row.creditPerMinute.toFixed(0),
      sorter: (a, b) => a.creditPerMinute - b.creditPerMinute,
      defaultSortOrder: "descend",
      align: "center",
    },
  ];
}

function createMissionTypeTableData(missionInfo: MissionInfo): MissionTypeInfoTableRow[] {
  let result = [];
  for (const [missionTypeId, missionTypeInfo] of Object.entries(missionInfo.missionTypeData)) {
    result.push({
      missionType: missionTypeId,
      mappedType:
        missionInfo.missionTypeMap[missionTypeId] === undefined
          ? missionTypeId
          : missionInfo.missionTypeMap[missionTypeId],
      ...missionTypeInfo,
    });
  }

  return result;
}

function createMissionTypePlot(missionInfo: MissionInfo) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 1,
      columns: 1,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let missionCountList: number[] = [];

  for (const [missionTypeId, missionTypeInfo] of Object.entries(missionInfo.missionTypeData)) {
    plotLabels.push(
      missionInfo.missionTypeMap[missionTypeId] === undefined
        ? missionTypeId
        : missionInfo.missionTypeMap[missionTypeId],
    );
    missionCountList.push(missionTypeInfo.missionCount);
  }

  let data: Data[] = [
    {
      values: missionCountList,
      labels: plotLabels,
      type: "pie",
      name: "任务数",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "任务数",
      },
    },
  ];

  Plotly.newPlot("mission-type-plot", data, plotLayout);
}

function createPlayerInfoSubTable(outerRow: PlayerInfoTableRow) {
  interface SubTableRow {
    characterGameId: string;
    characterCount: number;
  }
  const columns: DataTableColumns<SubTableRow> = [
    {
      title: "角色",
      key: "characterName",
      align: "center",
      render: (row) =>
        h(
          "span",
          { class: generateCharacterClass(row.characterGameId) },
          outerRow.characterMapping[row.characterGameId],
        ),
    },
    {
      title: "次数",
      key: "characterCount",
      align: "center",
      sorter: (a, b) => a.characterCount - b.characterCount,
      defaultSortOrder: "descend",
    },
  ];

  let data: SubTableRow[] = [];
  for (const [characterGameId, characterCount] of Object.entries(outerRow.characterInfo)) {
    data.push({
      characterGameId,
      characterCount,
    });
  }

  return h(NDataTable, {
    columns,
    data,
    pagination: false,
    style: "width: fit-content",
  });
}

function createPlayerInfoTableColumns(): DataTableColumns<PlayerInfoTableRow> {
  return [
    {
      type: "expand",
      renderExpand: (rowData) => createPlayerInfoSubTable(rowData),
    },
    {
      title: "玩家",
      key: "playerName",
      render: (row) => {
        let mostcharaterGameId = "";
        let mostCharacterCount = 0;
        for (const [charaterGameId, characterCount] of Object.entries(row.characterInfo)) {
          if (characterCount > mostCharacterCount) {
            mostcharaterGameId = charaterGameId;
            mostCharacterCount = characterCount;
          }
        }

        return h("span", { class: generateCharacterClass(mostcharaterGameId) }, row.playerName);
      },
      align: "center",
    },
    {
      title: "有效局数",
      key: "validMissionCount",
      sorter: (a, b) => a.validMissionCount - b.validMissionCount,
      defaultSortOrder: "descend",
      align: "center",
    },
    {
      title: "平均救人数",
      key: "averageReviveNum",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageReviveNum[0], total: row.averageReviveNum[1]},
        formatter: (x) => x.toFixed(2),
        style: "font-size: 1rem",
      }),
      sorter: (a, b) => a.averageReviveNum[1] - b.averageReviveNum[1],
      align: "center",
    },
    {
      title: "平均倒地数",
      key: "averageDeathNum",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageDeathNum[0], total: row.averageDeathNum[1]},
        formatter: (x) => x.toFixed(2),
        reverse: true,
        style: "font-size: 1rem",
      }),
      sorter: (a, b) => a.averageDeathNum[1] - b.averageDeathNum[1],
      align: "center",
    },
    {
      title: "平均采集量",
      key: "averageMineralsMined",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageMineralsMined[0], total: row.averageMineralsMined[1]},
        formatter: (x) => x.toFixed(2),
        style: "font-size: 1rem",
      }),
      sorter: (a, b) => a.averageMineralsMined[1] - b.averageMineralsMined[1],
      align: "center",
    },
    {
      title: "平均补给份数",
      key: "averageSupplyCount",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageSupplyCount[0], total: row.averageSupplyCount[1]},
        formatter: (x) => x.toFixed(2),
        reverse: true,
        style: "font-size: 1rem",
      }),
      sorter: (a, b) => a.averageSupplyCount[1] - b.averageSupplyCount[1],
      align: "center",
    },
    {
      title: "平均补给效率",
      key: "averageSupplyEfficiency",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageSupplyEfficiency[0], total: row.averageSupplyEfficiency[1]},
        formatter: formatPercent,
        style: "font-size: 1rem",
      }),
      sorter: (a, b) => a.averageSupplyEfficiency[1] - b.averageSupplyEfficiency[1],
      align: "center",
    },
  ];
}

function createPlayerInfoTableData(playerData: PlayerData): PlayerInfoTableRow[] {
  let result = [];
  for (const [playerName, playerInfo] of Object.entries(playerData.playerData)) {
    result.push({
      playerName,
      characterMapping: playerData.characterMap,
      averageDeathNum: [playerData.prevPlayerData[playerName].averageDeathNum, playerInfo.averageDeathNum],
      averageMineralsMined: [playerData.prevPlayerData[playerName].averageMineralsMined, playerInfo.averageMineralsMined],
      averageReviveNum: [playerData.prevPlayerData[playerName].averageReviveNum, playerInfo.averageReviveNum],
      averageSupplyCount: [playerData.prevPlayerData[playerName].averageSupplyCount, playerInfo.averageSupplyCount],
      averageSupplyEfficiency: [playerData.prevPlayerData[playerName].averageSupplyEfficiency, playerInfo.averageSupplyEfficiency],
      characterInfo: playerInfo.characterInfo,
      validMissionCount: playerInfo.validMissionCount,
    });
  }

  return result;
}

function createPlayerInfoPlot(playerData: PlayerData) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 2,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let characterLabels: string[] = [];
  let characterColorList: string[] = [];
  let playerLabels: string[] = [];
  let characterCountList: number[] = [];
  let playerMineralsMinedList: number[] = [];
  let playerDeathNumList: number[] = [];
  let playerSupplyCountList: number[] = [];

  if (characterInfoData.value === undefined) {
    return;
  }

  let characterCountMap: Record<string, number> = characterInfoData.value.characterChoiceCount;

  const characterToColor: Record<string, string> = {
    SCOUT: "#00BFFF",
    DRILLER: "#DAA520",
    ENGINEER: "#FF0000",
    GUNNER: "#008000",
  };

  for (const [characterGameId, characterCount] of Object.entries(characterCountMap)) {
    characterLabels.push(playerData.characterMap[characterGameId]);
    characterColorList.push(characterToColor[characterGameId]);
    characterCountList.push(characterCount);
  }

  for (const [playerName, playerInfo] of Object.entries(playerData.playerData)) {
    playerLabels.push(playerName);
    playerMineralsMinedList.push(playerInfo.averageMineralsMined);
    playerDeathNumList.push(playerInfo.averageDeathNum);
    playerSupplyCountList.push(playerInfo.averageSupplyCount);
  }

  let data: Data[] = [
    {
      values: characterCountList,
      labels: characterLabels,
      type: "pie",
      name: "角色选择次数",
      marker: {
        colors: characterColorList,
      },
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "角色选择次数",
      },
    },
    {
      values: playerMineralsMinedList,
      labels: playerLabels,
      type: "pie",
      name: "采集量",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "采集量",
      },
    },
    {
      values: playerDeathNumList,
      labels: playerLabels,
      type: "pie",
      name: "倒地数",
      domain: {
        row: 1,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "倒地数",
      },
    },
    {
      values: playerSupplyCountList,
      labels: playerLabels,
      type: "pie",
      name: "补给数",
      domain: {
        row: 1,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "补给数",
      },
    },
  ];

  Plotly.newPlot("player-info-plot", data, plotLayout);
}

function createCharacterInfoTableColumns(): DataTableColumns<CharacterInfoTableRow> {
  return [
    {
      title: "角色",
      key: "characterName",
      render: (row) => {
        return h("span", { class: generateCharacterClass(row.characterGameId) }, row.characterName);
      },
      align: "center",
    },
    {
      title: "总计玩家指数",
      key: "playerIndex",
      sorter: (a, b) => a.playerIndex - b.playerIndex,
      render: (row) => row.playerIndex.toFixed(2),
      defaultSortOrder: "descend",
      align: "center",
    },
    {
      title: "平均救人数",
      key: "reviveNum",
      render: (row) => row.reviveNum.toFixed(2),
      sorter: (a, b) => a.reviveNum - b.reviveNum,
      align: "center",
    },
    {
      title: "平均倒地数",
      key: "deathNum",
      render: (row) => row.deathNum.toFixed(2),
      sorter: (a, b) => a.deathNum - b.deathNum,
      align: "center",
    },
    {
      title: "平均采集量",
      key: "mineralsMined",
      render: (row) => nFormatter(row.mineralsMined, 2),
      sorter: (a, b) => a.mineralsMined - b.mineralsMined,
      align: "center",
    },
    {
      title: "平均补给份数",
      key: "supplyCount",
      render: (row) => row.supplyCount.toFixed(2),
      sorter: (a, b) => a.supplyCount - b.supplyCount,
      align: "center",
    },
    {
      title: "平均补给效率",
      key: "supplyEfficiency",
      render: (row) => formatPercent(row.supplyEfficiency),
      sorter: (a, b) => a.supplyEfficiency - b.supplyEfficiency,
      align: "center",
    },
  ];
}

function generateCharacterInfoTableData(
  characterInfo: CharacterGeneralInfo,
): CharacterInfoTableRow[] {
  let result = [];
  for (const [characterGameId, characterData] of Object.entries(characterInfo.characterData)) {
    result.push({
      characterGameId,
      characterName: characterInfo.characterMapping[characterGameId],
      ...characterData,
    });
  }

  return result;
}

function createCharacterInfoPlot(characterGeneralInfo: CharacterGeneralInfo) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 2,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let characterLabels: string[] = [];
  let characterColorList: string[] = [];
  let characterReviveNumList: number[] = [];
  let characterDeathNumList: number[] = [];
  let characterMineralsMinedList: number[] = [];
  let characterSupplyCountList: number[] = [];

  const characterToColor: Record<string, string> = {
    SCOUT: "#00BFFF",
    DRILLER: "#DAA520",
    ENGINEER: "#FF0000",
    GUNNER: "#008000",
  };

  for (const [characterGameId, characterData] of Object.entries(
    characterGeneralInfo.characterData,
  )) {
    characterLabels.push(characterGeneralInfo.characterMapping[characterGameId]);
    characterColorList.push(characterToColor[characterGameId]);
    characterReviveNumList.push(characterData.reviveNum);
    characterDeathNumList.push(characterData.deathNum);
    characterMineralsMinedList.push(characterData.mineralsMined);
    characterSupplyCountList.push(characterData.supplyCount);
  }

  let data: Data[] = [
    {
      values: characterReviveNumList,
      labels: characterLabels,
      type: "pie",
      name: "平均救人数",
      marker: {
        colors: characterColorList,
      },
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均救人数",
      },
    },
    {
      values: characterDeathNumList,
      labels: characterLabels,
      type: "pie",
      name: "平均倒地数",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均倒地数",
      },
    },
    {
      values: characterMineralsMinedList,
      labels: characterLabels,
      type: "pie",
      name: "平均采集量",
      domain: {
        row: 1,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均采集量",
      },
    },
    {
      values: characterSupplyCountList,
      labels: characterLabels,
      type: "pie",
      name: "平均补给份数",
      domain: {
        row: 1,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均补给份数",
      },
    },
  ];

  Plotly.newPlot("character-info-plot", data, plotLayout);
}

const message = useMessage();
const generalInfoData = ref<GeneralInfo>();
const missionInfoData = ref<MissionInfo>();
const playerInfoData = ref<PlayerData>();
const characterInfoData = ref<CharacterInfo>();
const characterGeneralInfo = ref<CharacterGeneralInfo>();

const missionTypeTableData = ref<MissionTypeInfoTableRow[]>([]);
const playerInfoTableData = ref<PlayerInfoTableRow[]>([]);
const characterInfoTableData = ref<CharacterInfoTableRow[]>([]);

fetch("./api/general/")
  .then((res) => res.json())
  .then((res: Response<GeneralInfo>) => {
    if (res.code !== 200) {
      message.error(`API Error: ${res.code} ${res.message}`);
    } else {
      generalInfoData.value = res.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error: ${err}`);
  });

fetch("./api/general/mission_type")
  .then((res) => res.json())
  .then((res: Response<MissionInfo>) => {
    if (res.code !== 200) {
      message.error(`API Error: ${res.code} ${res.message}`);
    } else {
      missionInfoData.value = res.data;
      missionTypeTableData.value = createMissionTypeTableData(res.data);
      createMissionTypePlot(res.data);
    }
  })
  .catch((err) => {
    message.error(`HTTP Error: ${err}`);
  });

fetch("./api/general/player")
  .then((res) => res.json())
  .then((res: Response<PlayerData>) => {
    if (res.code !== 200) {
      message.error(`API Error: ${res.code} ${res.message}`);
    } else {
      playerInfoData.value = res.data;
      playerInfoTableData.value = createPlayerInfoTableData(res.data);
      createPlayerInfoPlot(res.data);
    }
  })
  .catch((err) => {
    message.error(`HTTP Error: ${err}`);
  });

fetch("./api/general/character_info")
  .then((res) => res.json())
  .then((res: Response<CharacterInfo>) => {
    if (res.code !== 200) {
      message.error(`API Error: ${res.code} ${res.message}`);
    } else {
      characterInfoData.value = res.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error: ${err}`);
  });

fetch("./api/general/character")
  .then((res) => res.json())
  .then((res: Response<CharacterGeneralInfo>) => {
    if (res.code !== 200) {
      message.error(`API Error: ${res.code} ${res.message}`);
    } else {
      characterGeneralInfo.value = res.data;
      characterInfoTableData.value = generateCharacterInfoTableData(res.data);
      createCharacterInfoPlot(res.data);
    }
  })
  .catch((err) => {
    message.error(`HTTP Error: ${err}`);
  });

watch([() => characterInfoData.value, () => playerInfoData.value], () => {
  if (playerInfoData.value !== undefined) {
    createPlayerInfoPlot(playerInfoData.value);
  }
});
</script>
<template>
  <n-card title="任务">
    <n-grid :cols="4">
      <n-grid-item>
        <n-statistic label="游戏局数">{{ generalInfoData?.gameCount }}</n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="有效率">{{ formatPercent(generalInfoData?.validRate) }}</n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="总任务时长">{{
          formatMissionTime(generalInfoData?.totalMissionTime)
        }}</n-statistic>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均任务时长"
          :delta="generalInfoData?.averageMissionTime"
          :formatter="formatMissionTime"
          :reverse="true"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="独立玩家数">{{ generalInfoData?.uniquePlayerCount }}</n-statistic>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="野房率"
          :delta="generalInfoData?.openRoomRate"
          :formatter="formatPercent"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="通过率"
          :delta="generalInfoData?.passRate"
          :formatter="formatPercent"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均难度"
          :delta="generalInfoData?.averageDifficulty"
          :formatter="(total) => total.toFixed(2)"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均击杀数"
          :delta="generalInfoData?.averageKillNum"
          :formatter="(total) => total.toFixed(0)"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均伤害"
          :delta="generalInfoData?.averageDamage"
          :formatter="(total) => nFormatter(total, 2)"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均每人倒地数"
          :delta="generalInfoData?.averageDeathNumPerPlayer"
          :formatter="(total) => total.toFixed(2)"
          :reverse="true"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均采集量"
          :delta="generalInfoData?.averageMineralsMined"
          :formatter="(total) => nFormatter(total, 2)"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均每人补给份数"
          :delta="generalInfoData?.averageSupplyCountPerPlayer"
          :formatter="(total) => total.toFixed(2)"
          :reverse="true"
        ></DeltaView>
      </n-grid-item>
      <n-grid-item>
        <DeltaView
          label="平均奖励代币"
          :delta="generalInfoData?.averageRewardCredit"
          :formatter="(total) => nFormatter(total, 2)"
        ></DeltaView>
      </n-grid-item>
    </n-grid>
  </n-card>
  <n-card title="任务类型">
    <div class="table-plot-container">
      <n-data-table
        :columns="createMissionTypeTableColumns()"
        :data="missionTypeTableData"
        :pagination="false"
        :row-key="(row: MissionTypeInfoTableRow) => row.missionType"
        style="width: fit-content"
      >
      </n-data-table>
      <div id="mission-type-plot"></div>
    </div>
  </n-card>
  <n-card title="按角色">
    <div class="table-plot-container">
      <n-data-table
        :columns="createCharacterInfoTableColumns()"
        :data="characterInfoTableData"
        :pagination="false"
        :row-key="(row: CharacterInfoTableRow) => row.characterGameId"
        style="width: fit-content"
      ></n-data-table>
      <div id="character-info-plot"></div></div
  ></n-card>
  <n-card title="玩家">
    <div class="table-plot-container">
      <n-data-table
        :columns="createPlayerInfoTableColumns()"
        :data="playerInfoTableData"
        :pagination="false"
        :row-key="(row) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
      <div id="player-info-plot"></div>
    </div>
  </n-card>
</template>
<style scoped>
.table-plot-container {
  display: flex;
  justify-content: space-around;
  align-items: center;
}
</style>
