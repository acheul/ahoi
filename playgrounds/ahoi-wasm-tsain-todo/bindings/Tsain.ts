/*
 *  ████████╗███████╗ █████╗ ██╗███╗   ██╗
 *     ██╔══╝██╔════╝██╔══██╗██║████╗  ██║
 *     ██║   ███████╗███████║██║██╔██╗ ██║
 *     ██║   ╚════██║██╔══██║██║██║╚██╗██║
 *     ██║   ███████║██║  ██║██║██║ ╚████║
 *     ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝
 *
 *  Fast and Secure Connection of TypeScript & Rust
 *  ───────────────────────────────────────────────
 *  Auto-generated. Do not edit.
 */



// # Enum Tell
export type Tell = TellAddTodo | TellToggleTodo | TellRemoveTodo | TellClearDone;

// ## Var Ids
export const TellAddTodoId: number = 0;
export const TellToggleTodoId: number = 1;
export const TellRemoveTodoId: number = 2;
export const TellClearDoneId: number = 3;

// ## As Each Var
export const asTellAddTodo = (e: Tell): TellAddTodo | undefined => e[0] == 0 ? e as TellAddTodo : undefined;
export const asTellToggleTodo = (e: Tell): TellToggleTodo | undefined => e[0] == 1 ? e as TellToggleTodo : undefined;
export const asTellRemoveTodo = (e: Tell): TellRemoveTodo | undefined => e[0] == 2 ? e as TellRemoveTodo : undefined;
export const asTellClearDone = (e: Tell): TellClearDone | undefined => e[0] == 3 ? e as TellClearDone : undefined;

// ## Variant 0: TellAddTodo
// 1. Type
export type TellAddTodo = [0, [string]] & {readonly __brand: "TellAddTodo"};

// 2. Constructor
export const TellAddTodo_ = (f0: string) => [0, [f0]] as TellAddTodo;

// 3. Getters
export const TellAddTodo_f0_id: number = 0;
export const TellAddTodo_f0 = (v: TellAddTodo): string => v[1][0];


// ## Variant 1: TellToggleTodo
// 1. Type
export type TellToggleTodo = [1, [number]] & {readonly __brand: "TellToggleTodo"};

// 2. Constructor
export const TellToggleTodo_ = (f0: number) => [1, [f0]] as TellToggleTodo;

// 3. Getters
export const TellToggleTodo_f0_id: number = 0;
export const TellToggleTodo_f0 = (v: TellToggleTodo): number => v[1][0];


// ## Variant 2: TellRemoveTodo
// 1. Type
export type TellRemoveTodo = [2, [number]] & {readonly __brand: "TellRemoveTodo", readonly ret: Todo | undefined};

// 2. Constructor
export const TellRemoveTodo_ = (f0: number) => [2, [f0]] as TellRemoveTodo;

// 3. Getters
export const TellRemoveTodo_f0_id: number = 0;
export const TellRemoveTodo_f0 = (v: TellRemoveTodo): number => v[1][0];


// ## Variant 3: TellClearDone
// 1. Type
export type TellClearDone = [3, []] & {readonly __brand: "TellClearDone", readonly ret: number};

// 2. Constructor
export const TellClearDone_ = () => [3, []] as TellClearDone;



// # Enum Hail
export type Hail = HailUserName | HailFilter | HailTodos | HailOpenCount | HailMotto;

// ## Var Ids
export const HailUserNameId: number = 0;
export const HailFilterId: number = 1;
export const HailTodosId: number = 2;
export const HailOpenCountId: number = 3;
export const HailMottoId: number = 4;

// ## As Each Var
export const asHailUserName = (e: Hail): HailUserName | undefined => e[0] == 0 ? e as HailUserName : undefined;
export const asHailFilter = (e: Hail): HailFilter | undefined => e[0] == 1 ? e as HailFilter : undefined;
export const asHailTodos = (e: Hail): HailTodos | undefined => e[0] == 2 ? e as HailTodos : undefined;
export const asHailOpenCount = (e: Hail): HailOpenCount | undefined => e[0] == 3 ? e as HailOpenCount : undefined;
export const asHailMotto = (e: Hail): HailMotto | undefined => e[0] == 4 ? e as HailMotto : undefined;

// ## Variant 0: HailUserName
// 1. Type
export type HailUserName = [0, []] & {readonly __brand: "HailUserName", readonly ret: string};

// 2. Constructor
export const HailUserName_ = () => [0, []] as HailUserName;


// ## Variant 1: HailFilter
// 1. Type
export type HailFilter = [1, []] & {readonly __brand: "HailFilter", readonly ret: Filter};

// 2. Constructor
export const HailFilter_ = () => [1, []] as HailFilter;


// ## Variant 2: HailTodos
// 1. Type
export type HailTodos = [2, []] & {readonly __brand: "HailTodos", readonly ret: Todo[]};

// 2. Constructor
export const HailTodos_ = () => [2, []] as HailTodos;


// ## Variant 3: HailOpenCount
// 1. Type
export type HailOpenCount = [3, []] & {readonly __brand: "HailOpenCount", readonly ret: number};

// 2. Constructor
export const HailOpenCount_ = () => [3, []] as HailOpenCount;


// ## Variant 4: HailMotto
// 1. Type
export type HailMotto = [4, []] & {readonly __brand: "HailMotto", readonly ret: string};

// 2. Constructor
export const HailMotto_ = () => [4, []] as HailMotto;



// # Enum Pier
export type Pier = PierTop | PierUser;

// ## Var Ids
export const PierTopId: number = 0;
export const PierUserId: number = 1;

// ## As Each Var
export const asPierTop = (e: Pier): PierTop | undefined => e[0] == 0 ? e as PierTop : undefined;
export const asPierUser = (e: Pier): PierUser | undefined => e[0] == 1 ? e as PierUser : undefined;

// ## Variant 0: PierTop
// 1. Type
export type PierTop = [0, []] & {readonly __brand: "PierTop"};

// 2. Constructor
export const PierTop_ = () => [0, []] as PierTop;


// ## Variant 1: PierUser
// 1. Type
export type PierUser = [1, []] & {readonly __brand: "PierUser"};

// 2. Constructor
export const PierUser_ = () => [1, []] as PierUser;



// # Enum Filter
export type Filter = FilterAll | FilterOpen | FilterDone;

// ## Var Ids
export const FilterAllId: number = 0;
export const FilterOpenId: number = 1;
export const FilterDoneId: number = 2;

// ## As Each Var
export const asFilterAll = (e: Filter): FilterAll | undefined => e[0] == 0 ? e as FilterAll : undefined;
export const asFilterOpen = (e: Filter): FilterOpen | undefined => e[0] == 1 ? e as FilterOpen : undefined;
export const asFilterDone = (e: Filter): FilterDone | undefined => e[0] == 2 ? e as FilterDone : undefined;

// ## Variant 0: FilterAll
// 1. Type
export type FilterAll = [0, []] & {readonly __brand: "FilterAll"};

// 2. Constructor
export const FilterAll_ = () => [0, []] as FilterAll;


// ## Variant 1: FilterOpen
// 1. Type
export type FilterOpen = [1, []] & {readonly __brand: "FilterOpen"};

// 2. Constructor
export const FilterOpen_ = () => [1, []] as FilterOpen;


// ## Variant 2: FilterDone
// 1. Type
export type FilterDone = [2, []] & {readonly __brand: "FilterDone"};

// 2. Constructor
export const FilterDone_ = () => [2, []] as FilterDone;



// # Enum Status
export type Status = StatusOpen | StatusDone;

// ## Var Ids
export const StatusOpenId: number = 0;
export const StatusDoneId: number = 1;

// ## As Each Var
export const asStatusOpen = (e: Status): StatusOpen | undefined => e[0] == 0 ? e as StatusOpen : undefined;
export const asStatusDone = (e: Status): StatusDone | undefined => e[0] == 1 ? e as StatusDone : undefined;

// ## Variant 0: StatusOpen
// 1. Type
export type StatusOpen = [0, []] & {readonly __brand: "StatusOpen"};

// 2. Constructor
export const StatusOpen_ = () => [0, []] as StatusOpen;


// ## Variant 1: StatusDone
// 1. Type
export type StatusDone = [1, []] & {readonly __brand: "StatusDone"};

// 2. Constructor
export const StatusDone_ = () => [1, []] as StatusDone;



// # Struct Todo
// 1. Type
export type Todo = [number, string, Status] & {readonly __brand: "Todo"};

// 2. Constructor
export const Todo_ = (id: number, text: string, status: Status) => [id, text, status] as Todo;

// 3. Getters
export const Todo_id_id: number = 0;
export const Todo_id = (v: Todo): number => v[0];

export const Todo_text_id: number = 1;
export const Todo_text = (v: Todo): string => v[1];

export const Todo_status_id: number = 2;
export const Todo_status = (v: Todo): Status => v[2];


