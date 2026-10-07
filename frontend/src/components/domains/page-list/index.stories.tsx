import { Meta, StoryObj } from "@storybook/react";

import { PageList } from ".";

export default {
  component: PageList,
} as Meta;

export const Default: StoryObj = {
  render: () => {
    return <PageList />;
  },
};
