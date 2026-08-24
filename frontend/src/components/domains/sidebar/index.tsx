import { memo } from "react";

import { Nav } from "./styles";

import { PageList } from "@/components/domains/page-list";

export const Sidebar = memo(() => {
  return (
    <Nav>
      <PageList />
    </Nav>
  );
});
