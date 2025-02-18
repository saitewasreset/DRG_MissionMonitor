<script setup lang="ts">
import { ref, h, watch } from "vue";
import { useMessage } from "naive-ui";
import { NDataTable, NCard, type DataTableColumns } from "naive-ui";
import { generateCharacterClass } from "@/formatter";
import Plotly from "plotly.js-basic-dist";
import type { Data } from "plotly.js-basic-dist";
import { translate } from "@/mapping";
import type { Response } from "@/type";

type PlayerBasicInfo = Record<string, string>;

interface PlayerResourceInfo {
  resource: Record<string, number>;
  supply: {
    ammo: number;
    health: number;
  }[];
}

type ResourceData = Record<string, PlayerResourceInfo>;

interface NitraDataRow {
  playerName: string;
  nitra: number;
  supplyCount: number;
  averageUtilization: number;
  netNitra: number;
}

interface ResourceDataRow {
  resourceName: string;
  totalAmount: number;
}

interface PlayerResourceDataRow {
  playerName: string;
  amount: number;
  children?: PlayerResourceDataRow[];
}

function createNitraTableColumns(): DataTableColumns<NitraDataRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(playerCharacterMap.value[row.playerName]) },
          row.playerName,
        );
      },
    },
    {
      title: "硝石采集量",
      key: "nitra",
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.nitra - b.nitra;
      },
      render(row) {
        return row.nitra.toFixed(0);
      },
    },
    {
      title: "补给份数",
      key: "supplyCount",
      align: "center",
      sorter(a, b) {
        return a.supplyCount - b.supplyCount;
      },
    },
    {
      title: "平均补给效率",
      key: "averageUtilization",
      align: "center",
      sorter(a, b) {
        return a.averageUtilization - b.averageUtilization;
      },
      render(row) {
        return `${(row.averageUtilization * 100).toFixed(2)}%`;
      },
    },
    {
      title: "净硝石量",
      key: "netNitra",
      align: "center",
      sorter(a, b) {
        return a.netNitra - b.netNitra;
      },
      render(row) {
        return row.netNitra.toFixed(0);
      },
    },
  ];
}

function generateNitraTableData(data: ResourceData): NitraDataRow[] {
  let result = [];
  for (const [playerName, playerData] of Object.entries(data)) {
    let nitra = 0.0;
    if ("RES_VEIN_Nitra" in playerData.resource) {
      nitra = playerData.resource["RES_VEIN_Nitra"];
    }
    const supplyCount = playerData.supply.length;
    let averageUtilization = 1.0;

    if (supplyCount > 0) {
      let sum = 0.0;
      for (const supplyInfo of playerData.supply) {
        sum += supplyInfo.ammo;
      }
      averageUtilization = sum / supplyCount / 0.5;
    }

    const netNitra = nitra - supplyCount * 20;
    result.push({
      playerName,
      nitra,
      supplyCount,
      averageUtilization,
      netNitra,
    });
  }

  return result;
}

function createNitraPlot(resourceData: ResourceData) {
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
  let nitraList: number[] = [];
  let supplyCountList: number[] = [];

  for (const [playerName, playerData] of Object.entries(resourceData)) {
    plotLabels.push(playerName);
    let nitra = 0.0;
    if ("RES_VEIN_Nitra" in playerData.resource) {
      nitra = playerData.resource["RES_VEIN_Nitra"];
    }
    nitraList.push(nitra);

    const supplyCount = playerData.supply.length;
    supplyCountList.push(supplyCount);
  }

  let data: Data[] = [
    {
      values: nitraList,
      labels: plotLabels,
      type: "pie",
      name: "硝石采集量",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "硝石采集量",
      },
    },
    {
      values: supplyCountList,
      labels: plotLabels,
      type: "pie",
      name: "补给份数",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "补给份数",
      },
    },
  ];

  Plotly.newPlot("nitra-plot-container", data, plotLayout);
}

function createResourceTableColumns(): DataTableColumns<ResourceDataRow> {
  return [
    {
      title: "资源",
      key: "resourceName",
      align: "center",
      render(row) {
        return translate(row.resourceName).value;
      },
    },
    {
      title: "总采集量",
      key: "totalAmount",
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.totalAmount - b.totalAmount;
      },
      render(row) {
        return row.totalAmount.toFixed(0);
      },
    },
  ];
}

function generateResourceTableData(data: ResourceData): ResourceDataRow[] {
  let resourceMap: Record<string, number> = {};

  let result = [];

  for (const playerData of Object.values(data)) {
    for (const [resourceName, amount] of Object.entries(playerData.resource)) {
      if (resourceName in resourceMap) {
        resourceMap[resourceName] += amount;
      } else {
        resourceMap[resourceName] = amount;
      }
    }
  }

  for (const [resourceName, totalAmount] of Object.entries(resourceMap)) {
    result.push({ resourceName, totalAmount });
  }

  return result;
}

function createResourcePlot(resourceData: ResourceData, mapping: Record<string, string>) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let amountList: number[] = [];

  let resourceMap: Record<string, number> = {};

  for (const playerData of Object.values(resourceData)) {
    for (const [resourceName, amount] of Object.entries(playerData.resource)) {
      if (resourceName in resourceMap) {
        resourceMap[resourceName] += amount;
      } else {
        resourceMap[resourceName] = amount;
      }
    }
  }

  for (const [resourceName, totalAmount] of Object.entries(resourceMap)) {
    plotLabels.push(resourceName in mapping ? mapping[resourceName] : resourceName);
    amountList.push(totalAmount);
  }

  let data: Data[] = [
    {
      values: amountList,
      labels: plotLabels,
      type: "pie",
      name: "矿物采集量",
      hoverinfo: "label+value+percent",
      title: {
        text: "矿物采集量",
      },
    },
  ];

  Plotly.newPlot("resource-plot-container", data, plotLayout);
}

function createPlayerResourceTableColumn(): DataTableColumns<PlayerResourceDataRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(playerCharacterMap.value[row.playerName]) },
          row.playerName,
        );
      },
    },
    {
      title: "总采集量",
      key: "amount",
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.amount - b.amount;
      },
      render(row) {
        return row.amount.toFixed(0);
      },
    },
  ];
}

function generatePlayerResourceTableData(data: ResourceData): PlayerResourceDataRow[] {
  let result = [];
  for (const [playerName, playerData] of Object.entries(data)) {
    let childrenRowList: PlayerResourceDataRow[] = [];
    let totalAmount = 0;
    for (const [resourceGameId, amount] of Object.entries(playerData.resource)) {
      childrenRowList.push({ playerName: translate(resourceGameId).value, amount });
      totalAmount += amount;
    }
    result.push({
      playerName,
      amount: totalAmount,
      children: childrenRowList,
    });
  }

  return result;
}

function createPlayerResourcePlot(resourceData: ResourceData) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let amountList: number[] = [];

  let playerMap: Record<string, number> = {};

  for (const [playerName, playerData] of Object.entries(resourceData)) {
    for (const amount of Object.values(playerData.resource)) {
      if (playerName in playerMap) {
        playerMap[playerName] += amount;
      } else {
        playerMap[playerName] = amount;
      }
    }
  }

  for (const [playerName, totalAmount] of Object.entries(playerMap)) {
    plotLabels.push(playerName);
    amountList.push(totalAmount);
  }

  let data: Data[] = [
    {
      values: amountList,
      labels: plotLabels,
      type: "pie",
      name: "矿物采集量",
      hoverinfo: "label+value+percent",
      title: {
        text: "矿物采集量",
      },
    },
  ];

  Plotly.newPlot("player-resource-plot-container", data, plotLayout);
}

const props = defineProps<{ missionId?: number }>();
const message = useMessage();

const playerCharacterMap = ref<Record<string, string>>({});
const resourceData = ref<ResourceData>({});

const nitraTableData = ref<NitraDataRow[]>([]);
const resourceTableData = ref<ResourceDataRow[]>([]);

const playerResourceTableData = ref<PlayerResourceDataRow[]>([]);

watch(
  () => props.missionId,
  () => {
    fetch(`./api/mission/${props.missionId == undefined ? 1 : props.missionId}/basic`)
      .then((res) => res.json())
      .then((data: Response<PlayerBasicInfo>) => {
        if (data.code !== 200) {
          message.error(`API Error: ${data.code} ${data.message}`);
        } else {
          playerCharacterMap.value = data.data;
        }
      })
      .catch((e) => message.error(`HTTP error: ${e}`));

    fetch(`./api/mission/${props.missionId == undefined ? 1 : props.missionId}/resource`)
      .then((res) => res.json())
      .then((data: Response<{ data: ResourceData; resourceMapping: Record<string, string> }>) => {
        if (data.code !== 200) {
          message.error(`API Error: ${data.code} ${data.message}`);
        } else {
          resourceData.value = data.data.data;

          nitraTableData.value = generateNitraTableData(data.data.data);
          createNitraPlot(data.data.data);

          resourceTableData.value = generateResourceTableData(data.data.data);
          createResourcePlot(data.data.data, data.data.resourceMapping);

          playerResourceTableData.value = generatePlayerResourceTableData(data.data.data);
          createPlayerResourcePlot(data.data.data);
        }
      })
      .catch((e) => message.error(`HTTP error: ${e}`));
  },
  { immediate: true },
);
</script>

<template>
  <n-card title="硝石与补给">
    <div class="table-plot-container">
      <n-data-table
        :columns="createNitraTableColumns()"
        :data="nitraTableData"
        :pagination="false"
        :row-key="(row: NitraDataRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
      <div id="nitra-plot-container"></div>
    </div>
  </n-card>
  <n-card title="资源种类">
    <div class="table-plot-container">
      <n-data-table
        :columns="createResourceTableColumns()"
        :data="resourceTableData"
        :pagination="{ pageSize: 10 }"
        :row-key="(row: ResourceDataRow) => row.resourceName"
        style="width: fit-content"
      ></n-data-table>
      <div id="resource-plot-container"></div>
    </div>
  </n-card>
  <n-card title="资源采集">
    <div class="table-plot-container">
      <n-data-table
        :columns="createPlayerResourceTableColumn()"
        :data="playerResourceTableData"
        :pagination="{ pageSize: 10 }"
        :row-key="(row: PlayerResourceDataRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
      <div id="player-resource-plot-container"></div>
    </div>
  </n-card>
</template>

<style scoped>
.table-plot-container {
  display: flex;
  justify-content: space-around;
  align-items: center;
}
h2 {
  text-align: center;
}
</style>
