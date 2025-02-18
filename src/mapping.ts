import { computed, ref } from "vue";

interface MappingResponse {
  code: number;
  message: string;
  data: Record<string, Record<string, string>>;
}

const mappingUrl = "./api/mapping";

export const mappingData = ref<Record<string, Record<string, string>> | null>(null);
export const mappingError = ref<string | null>(null);

fetch(mappingUrl)
  .then((data) => data.json())
  .then((json: MappingResponse) => {
    if (json.code !== 200) {
      mappingError.value = `API error: ${json.code} ${json.message}`;
    } else {
      mappingData.value = json.data;
    }
  })
  .catch((reason) => {
    mappingError.value = `HTTP error: ${reason}`;
  });

export function translate(source: string | undefined) {
  return computed(() => {
    if (mappingData.value === null) {
      if (source === undefined) {
        return "";
      } else {
        return source;
      }
    } else {
      const mappingList = ["character", "entity", "missionType", "resource", "weapon"];
      const combinedMapping: Record<string, string> = {};
      for (const mappingName of mappingList) {
        for (const sourceName in mappingData.value[mappingName]) {
          combinedMapping[sourceName] = mappingData.value[mappingName][sourceName];
        }
      }

      if (source === undefined) {
        return "";
      } else if (source in combinedMapping) {
        return combinedMapping[source];
      } else {
        return source;
      }
    }
  });
}
