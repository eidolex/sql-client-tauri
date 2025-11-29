import type { ColumnDefinition } from "$lib/db";

type BaseTab = {
  id: string;
  title: string;
  connectionId: string;
  database: string;
};

export type TableTab = BaseTab & {
  type: "data" | "structure";
  table: string;
  columns?: ColumnDefinition[];
  page: number;
  pageSize: number;
  totalRows: number;
  data?: any[];
};

export type QueryTab = BaseTab & {
  type: "query";
  query?: string;
};

export type Tab = TableTab | QueryTab;

export class TableTabState {
  items = $state<Tab[]>([]);

  addTab(tab: Tab) {
    this.items.push(tab);
  }

  removeTab(id: string) {
    this.items.splice(
      this.items.findIndex((t) => t.id === id),
      1
    );
  }

  getTab(id: string) {
    return this.items.find((t) => t.id === id);
  }
}
