import { defineStore } from "pinia";

interface CheckSessionResponse {
  code: number;
  message: string;
}

export const useLoginStore = defineStore("login", {
  state: () => {
    return { login: false };
  },

  actions: {
    checkLogin() {
      fetch(`./api/check_session`)
        .then((res) => res.json())
        .then((data: CheckSessionResponse) => {
          if (data.code == 200) {
            this.login = true;
          } else if (data.code == 403) {
            this.login = false;
          }
        });
    },
  },
});
