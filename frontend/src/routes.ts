const client = "/client";

export const routes = {
  channel: {
    show: (workspaceId: string, channelId: string) =>
      `${client}/${workspaceId}/${channelId}`,
  },
  notion: {
    page: {
      show: (pageId: string) => `/${pageId}`,
    },
  },
};
