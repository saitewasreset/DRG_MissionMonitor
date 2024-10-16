<script setup lang="ts">
import type { OverallDamageInfo } from "./DamageViewTypes";
import { NDataTable, NCard, NSelect, type DataTableColumns, type SelectOption } from "naive-ui";
import Plotly, { type Data } from "plotly.js-basic-dist";
import { nFormatter } from "@/formatter";
import { ref, watch, h } from "vue";
import DeltaView from "./DeltaView.vue";

interface FriendlyFireTableRow {
  playerName: string;
  validGameCount: number;
  averageDamage: number[];
  averageCause: number[];
  averageTake: number[];
  friendlyFireRate: number[];
  heartbrokenRate: number[];
}

interface PlayerCauseRow {
  takePlayerName: string;
  totalDamage: number;
  validGameCount: number;
  averageDamage: number;
}

interface PlayerTakeRow {
  causePlayerName: string;
  totalDamage: number;
  validGameCount: number;
  averageDamage: number;
}

function heartbrokenRateToClass(rate: number) {
  if (rate >= 1) {
    return "text-ok";
  } else if (rate >= 0.8) {
    return "text-warning";
  } else {
    return "text-danger";
  }
}

function createFriendlyFireTableColumns(): DataTableColumns<FriendlyFireTableRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      align: "center",
    },
    {
      title: "有效局数",
      key: "validGameCount",
      align: "center",
      sorter: (a, b) => a.validGameCount - b.validGameCount,
    },
    {
      title: "平均伤害",
      key: "averageDamage",
      align: "center",
      sorter: (a, b) => a.averageDamage[1] - b.averageDamage[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageDamage[0], total: row.averageDamage[1]},
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }),
    },
    {
      title: "平均造成友伤",
      key: "averageCause",
      align: "center",
      sorter: (a, b) => a.averageCause[1] - b.averageCause[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageCause[0], total: row.averageCause[1]},
        reverse: true,
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }),
    },
    {
      title: "平均受到友伤",
      key: "averageTake",
      align: "center",
      sorter: (a, b) => a.averageTake[1] - b.averageTake[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageTake[0], total: row.averageTake[1]},
        reverse: true,
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }),
    },
    {
      title: "友伤比例",
      key: "friendlyFireRate",
      align: "center",
      sorter: (a, b) => a.friendlyFireRate[1] - b.friendlyFireRate[1],
      defaultSortOrder: "descend",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.friendlyFireRate[0], total: row.friendlyFireRate[1]},
        reverse: true,
        formatter: (x) => (x * 1000).toFixed(2) + "‰",
        style: "font-size: 1rem",
      })
    },
    {
      title: "心碎值",
      key: "heartbrokenRate",
      align: "center",
      sorter: (a, b) => a.heartbrokenRate[1] - b.heartbrokenRate[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.heartbrokenRate[0], total: row.heartbrokenRate[1]},
        formatter: (x) => x.toFixed(2),
        style: "font-size: 1rem",
        inner_class: `${heartbrokenRateToClass(row.heartbrokenRate[1])}`,
      })
    },
  ];
}

function generateFriendlyFireTableData(): FriendlyFireTableRow[] {
  if (props.overallDamageInfo === undefined) {
    return [];
  }

  let result = [];

  for (const [playerName, playerDamageInfo] of Object.entries(props.overallDamageInfo.info)) {

    let prevInfo = props.overallDamageInfo.prevInfo[playerName];

    let totalDamage = 0.0;
    let prevDamage = 0.0;
    
    for (const damage of Object.values(playerDamageInfo.damage)) {
      totalDamage += damage;
    }

    for (const damage of Object.values(prevInfo.damage)) {
      prevDamage += damage;
    }

    let totalCause = 0.0;
    let totalTake = 0.0;

    let prevCause = 0.0;
    let prevTake = 0.0;

    for (const cause of Object.values(playerDamageInfo.ff.cause)) {
      totalCause += cause.damage;
    }

    for (const take of Object.values(playerDamageInfo.ff.take)) {
      totalTake += take.damage;
    }

    for (const cause of Object.values(prevInfo.ff.cause)) {
      prevCause += cause.damage;
    }

    for (const take of Object.values(prevInfo.ff.take)) {
      prevTake += take.damage;
    }

    result.push({
      playerName,
      validGameCount: playerDamageInfo.validGameCount,
      averageDamage: [prevDamage / prevInfo.validGameCount, totalDamage / playerDamageInfo.validGameCount],
      averageCause: [prevCause / prevInfo.validGameCount, totalCause / playerDamageInfo.validGameCount],
      averageTake: [prevTake / prevInfo.validGameCount, totalTake / playerDamageInfo.validGameCount],
      friendlyFireRate: [prevCause / prevDamage, totalCause / totalDamage],
      heartbrokenRate: [prevTake / prevCause, totalTake / totalCause],
    });
  }

  return result;
}

function createPlot() {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 1,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let totalCauseList: number[] = [];
  let totalTakeList: number[] = [];
  let heartBrokenList: { playerName: string; value: number }[] = [];

  if (props.overallDamageInfo === undefined) {
    return;
  }

  for (const [playerName, playerDamageInfo] of Object.entries(props.overallDamageInfo.info)) {
    plotLabels.push(playerName);
    let totalCause = 0.0;
    let totalTake = 0.0;

    for (const cause of Object.values(playerDamageInfo.ff.cause)) {
      totalCause += cause.damage;
    }

    for (const take of Object.values(playerDamageInfo.ff.take)) {
      totalTake += take.damage;
    }

    totalCauseList.push(totalCause / playerDamageInfo.validGameCount);
    totalTakeList.push(totalTake / playerDamageInfo.validGameCount);
    heartBrokenList.push({
      playerName,
      value: totalTake / totalCause,
    });
  }

  heartBrokenList.sort((a, b) => a.value - b.value);

  let heartBrokenLabels: string[] = [];
  let heartBrokenValues: number[] = [];

  for (const { playerName, value } of heartBrokenList) {
    heartBrokenLabels.push(playerName);
    heartBrokenValues.push(value);
  }

  let friendlyFireData: Data[] = [
    {
      values: totalCauseList,
      labels: plotLabels,
      type: "pie",
      name: "平均造成友伤",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均造成友伤",
      },
    },
    {
      values: totalTakeList,
      labels: plotLabels,
      type: "pie",
      name: "平均受到友伤",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均受到友伤",
      },
    },
  ];

  Plotly.newPlot("friendly-fire-plot", friendlyFireData, plotLayout);
  Plotly.newPlot(
    "hearrtbroken-plot",
    [
      {
        x: heartBrokenValues,
        y: heartBrokenLabels,
        type: "bar",
        orientation: "h",
      },
    ],
    {
      title: "心碎值",
      paper_bgcolor: "rgba(255,255,255, 0)",
      plot_bgcolor: "rgba(255,255,255, 0)",
    },
  );
}

function initSelect() {
  if (props.overallDamageInfo === undefined) {
    return;
  }

  for (const playerName of Object.keys(props.overallDamageInfo.info)) {
    playerList.value.push(playerName);
    selectOptions.value.push({
      label: playerName,
      value: playerName,
    });
  }

  currentSelectedPlayer.value = playerList.value[0];
}

function createCauseTableColumns(): DataTableColumns<PlayerCauseRow> {
  return [
    {
      title: "承受玩家",
      key: "takePlayerName",
      align: "center",
    },
    {
      title: "总伤害",
      key: "totalDamage",
      align: "center",
      sorter: (a, b) => a.totalDamage - b.totalDamage,
      render: (row) => nFormatter(row.totalDamage, 2),
    },
    {
      title: "有效局数",
      key: "validGameCount",
      align: "center",
      sorter: (a, b) => a.validGameCount - b.validGameCount,
    },
    {
      title: "平均伤害",
      key: "averageDamage",
      align: "center",
      sorter: (a, b) => a.averageDamage - b.averageDamage,
      render: (row) => row.averageDamage.toFixed(2),
      defaultSortOrder: "descend",
    },
  ];
}

function createTakeTableColumns(): DataTableColumns<PlayerTakeRow> {
  return [
    {
      title: "造成玩家",
      key: "causePlayerName",
      align: "center",
    },
    {
      title: "总伤害",
      key: "totalDamage",
      align: "center",
      sorter: (a, b) => a.totalDamage - b.totalDamage,
      render: (row) => nFormatter(row.totalDamage, 2),
    },
    {
      title: "有效局数",
      key: "validGameCount",
      align: "center",
      sorter: (a, b) => a.validGameCount - b.validGameCount,
    },
    {
      title: "平均伤害",
      key: "averageDamage",
      align: "center",
      sorter: (a, b) => a.averageDamage - b.averageDamage,
      render: (row) => row.averageDamage.toFixed(2),
      defaultSortOrder: "descend",
    },
  ];
}

function generateTakeTableData(): PlayerTakeRow[] {
  if (props.overallDamageInfo === undefined) {
    return [];
  }

  let result = [];

  const selectedPlayerName = currentSelectedPlayer.value;
  if (selectedPlayerName === null) {
    return [];
  }

  for (const [playerName, playerDamageInfo] of Object.entries(props.overallDamageInfo.info)) {
    if (playerName === selectedPlayerName) {
      for (const [causePlayerName, takeDamage] of Object.entries(playerDamageInfo.ff.take)) {
        if (takeDamage.gameCount === 0) {
          continue;
        }
        result.push({
          causePlayerName,
          totalDamage: takeDamage.damage,
          validGameCount: takeDamage.gameCount,
          averageDamage: takeDamage.damage / takeDamage.gameCount,
        });
      }
    }
  }

  return result;
}

function generateCauseTableData(): PlayerCauseRow[] {
  if (props.overallDamageInfo === undefined) {
    return [];
  }

  let result = [];

  const selectedPlayerName = currentSelectedPlayer.value;
  if (selectedPlayerName === null) {
    return [];
  }

  for (const [playerName, playerDamageInfo] of Object.entries(props.overallDamageInfo.info)) {
    if (playerName === selectedPlayerName) {
      for (const [takePlayerName, causeDamage] of Object.entries(playerDamageInfo.ff.cause)) {
        if (causeDamage.gameCount === 0) {
          continue;
        }
        result.push({
          takePlayerName,
          totalDamage: causeDamage.damage,
          validGameCount: causeDamage.gameCount,
          averageDamage: causeDamage.damage / causeDamage.gameCount,
        });
      }
    }
  }

  return result;
}

function createCausePlot() {
  const layout = {
    height: 400,
    width: 600,
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };
  if (props.overallDamageInfo === undefined) {
    return;
  }

  const selectedPlayerName = currentSelectedPlayer.value;
  if (selectedPlayerName === null) {
    return;
  }

  let plotLabels: string[] = [];
  let totalCauseList: number[] = [];

  for (const [takePlayerName, takeDamage] of Object.entries(
    props.overallDamageInfo.info[selectedPlayerName].ff.cause,
  )) {
    if (takeDamage.gameCount === 0) {
      continue;
    }
    plotLabels.push(takePlayerName);
    totalCauseList.push(takeDamage.damage / takeDamage.gameCount);
  }

  let causeData: Data[] = [
    {
      labels: plotLabels,
      values: totalCauseList,
      type: "pie",
      hoverinfo: "label+value+percent",
      title: {
        text: "该玩家造成友伤",
      },
    },
  ];

  Plotly.newPlot("cause-plot", causeData, layout);
}

function createTakePlot() {
  const layout = {
    height: 400,
    width: 600,
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };
  if (props.overallDamageInfo === undefined) {
    return;
  }

  const selectedPlayerName = currentSelectedPlayer.value;
  if (selectedPlayerName === null) {
    return;
  }

  let plotLabels: string[] = [];
  let totalTakeList: number[] = [];

  for (const [causePlayerName, causeDamage] of Object.entries(
    props.overallDamageInfo.info[selectedPlayerName].ff.take,
  )) {
    if (causeDamage.gameCount === 0) {
      continue;
    }
    plotLabels.push(causePlayerName);
    totalTakeList.push(causeDamage.damage / causeDamage.gameCount);
  }

  let takeData: Data[] = [
    {
      labels: plotLabels,
      values: totalTakeList,
      type: "pie",
      hoverinfo: "label+value+percent",
      title: {
        text: "该玩家受到友伤",
      },
    },
  ];

  Plotly.newPlot("take-plot", takeData, layout);
}

const props = defineProps<{
  overallDamageInfo?: OverallDamageInfo;
}>();

const friendlyFireTableData = ref<FriendlyFireTableRow[]>([]);

const playerList = ref<string[]>([]);
const currentSelectedPlayer = ref<string | null>(null);
const selectOptions = ref<SelectOption[]>([]);

const causeTableData = ref<PlayerCauseRow[]>([]);
const takeTableData = ref<PlayerTakeRow[]>([]);

watch(
  () => props.overallDamageInfo,
  () => {
    friendlyFireTableData.value = generateFriendlyFireTableData();
    createPlot();
    initSelect();
    takeTableData.value = generateTakeTableData();
    causeTableData.value = generateCauseTableData();
    createTakePlot();
    createCausePlot();
  },
);

watch(
  () => currentSelectedPlayer.value,
  () => {
    takeTableData.value = generateTakeTableData();
    causeTableData.value = generateCauseTableData();
    createTakePlot();
    createCausePlot();
  },
);
</script>
<template>
  <n-card title="总体">
    <div class="table-plot-container">
      <n-data-table
        :columns="createFriendlyFireTableColumns()"
        :data="friendlyFireTableData"
        :pagination="false"
        :row-key="(row: FriendlyFireTableRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
    </div>
    <div class="table-plot-container">
      <div id="friendly-fire-plot"></div>
      <div id="hearrtbroken-plot"></div>
    </div>
  </n-card>
  <n-card title="按玩家">
    <n-select
      v-model:value="currentSelectedPlayer"
      :options="selectOptions"
      style="max-width: 250px"
    ></n-select>
    <div>
      <n-card title="受到友伤">
        <div class="table-plot-container">
          <n-data-table
            :columns="createTakeTableColumns()"
            :data="takeTableData"
            :pagination="false"
            :row-key="(row: PlayerTakeRow) => row.causePlayerName"
            style="width: fit-content"
          ></n-data-table>
          <div id="take-plot"></div>
        </div>
      </n-card>
      <n-card title="造成友伤">
        <div class="table-plot-container">
          <n-data-table
            :columns="createCauseTableColumns()"
            :data="causeTableData"
            :pagination="false"
            :row-key="(row: PlayerCauseRow) => row.takePlayerName"
            style="width: fit-content"
          ></n-data-table>
          <div id="cause-plot"></div>
        </div>
      </n-card>
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
