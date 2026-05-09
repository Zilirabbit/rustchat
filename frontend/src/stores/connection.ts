import { defineStore } from "pinia";

export const useConnectionStore = defineStore("connection", {
  state: () => ({
    connected: false,
  }),
  actions: {
    setConnected(connected: boolean) {
      this.connected = connected;
    },
  },
});
