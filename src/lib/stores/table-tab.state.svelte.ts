import type { ColumnDefinition } from "$lib/db";

type IBaseTab = {
  id: string;
  title: string;
  connectionId: string;
  database: string;
  active: boolean;
};

export type ITableDataTab = {
  type: "data" | "structure";
  data: {
    table: string;
    columns?: ColumnDefinition[];
    page: number;
    pageSize: number;
    totalRows: number;
    rows?: any[];
  };
};

export type IQueryTab = {
  type: "query";
  data: {
    query?: string;
  };
};

export type TableTabOptions = IBaseTab & (ITableDataTab | IQueryTab);

export type TableTabType = TableTabOptions["type"];

type ChangedCallback = () => void | Promise<void>;

export class TableDataTab {
  #table: string;
  #columns?: ColumnDefinition[] = $state<ColumnDefinition[]>([]);
  #page: number = $state(1);
  #pageSize: number = $state(50);
  #totalRows: number = $state(0);
  #rows?: any[] = $state<any[]>([]);

  #changed: ChangedCallback;

  constructor(data: ITableDataTab["data"], changed: ChangedCallback) {
    this.#table = data.table;
    this.#columns = data.columns;
    this.#page = data.page;
    this.#pageSize = data.pageSize;
    this.#totalRows = data.totalRows;
    this.#rows = data.rows;
    this.#changed = changed;
  }

  get table() {
    return this.#table;
  }

  get columns() {
    return this.#columns;
  }

  get page() {
    return this.#page;
  }

  get pageSize() {
    return this.#pageSize;
  }

  get totalRows() {
    return this.#totalRows;
  }

  get rows() {
    return this.#rows;
  }

  set columns(columns: ColumnDefinition[] | undefined) {
    this.#columns = columns;
    this.#changed();
  }

  set page(page: number) {
    this.#page = page;
    this.#changed();
  }

  set pageSize(pageSize: number) {
    this.#pageSize = pageSize;
    this.#changed();
  }

  set totalRows(totalRows: number) {
    this.#totalRows = totalRows;
    this.#changed();
  }

  set rows(rows: any[] | undefined) {
    this.#rows = rows;
    this.#changed();
  }
}

export class TableQueryTab {
  #query?: string;
  #changed: ChangedCallback;

  constructor(data: IQueryTab["data"], changed: ChangedCallback) {
    this.#query = data.query;
    this.#changed = changed;
  }

  get query() {
    return this.#query;
  }

  set query(query: string | undefined) {
    this.#query = query;
    this.#changed();
  }
}

type TabDataType<T extends TableTabType> = T extends "data" | "structure"
  ? TableDataTab
  : T extends "query"
  ? TableQueryTab
  : never;

type TabOptionsType<T extends TableTabType> = IBaseTab &
  (T extends "data" | "structure"
    ? ITableDataTab & { type: T }
    : IQueryTab & { type: T });

export class TableTab<T extends TableTabType> {
  #id: string;
  #title: string;
  #connectionId: string;
  #database: string;
  #type: T = $state<T>("data" as T);

  #data: TabDataType<T>;

  constructor(data: TabOptionsType<T>) {
    this.#id = data.id;
    this.#title = data.title;
    this.#connectionId = data.connectionId;
    this.#database = data.database;
    this.#type = data.type as T;
    if (data.type === "data" || data.type === "structure") {
      this.#data = new TableDataTab(
        data.data,
        this.save.bind(this)
      ) as TabDataType<T>;
    } else {
      this.#data = new TableQueryTab(
        data.data,
        this.save.bind(this)
      ) as TabDataType<T>;
    }
  }

  async save() {}

  get id() {
    return this.#id;
  }

  get title() {
    return this.#title;
  }

  get connectionId() {
    return this.#connectionId;
  }

  get database() {
    return this.#database;
  }

  get type(): T {
    return this.#type;
  }

  set type(type: Exclude<T, "query">) {
    this.#type = type;
    this.save();
  }

  get data(): TabDataType<T> {
    return this.#data;
  }
}

// export class TableTabState {
//   items = $state<Tab[]>([]);

//   addTab(tab: Tab) {
//     this.items.push(tab);
//   }

//   removeTab(id: string) {
//     this.items.splice(
//       this.items.findIndex((t) => t.id === id),
//       1
//     );
//   }

//   getTab(id: string) {
//     return this.items.find((t) => t.id === id);
//   }
// }
