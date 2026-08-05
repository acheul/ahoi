/**
 * A small todo app over the tsain bridge. Values cross the wasm boundary in
 * tsain's positional array format, so components build keys with the
 * generated factories (`HailTodos_()`, `TellAddTodo_(text)`) and read data
 * with the generated getters (`Todo_text(t)`, `asStatusDone(s)`).
 */
import { For, Show, createMemo, createSignal } from "solid-js";
import { PierProvider, usePier } from "./bridge";
import {
    type Filter,
    type Todo,
    FilterAll_,
    FilterDone_,
    FilterOpen_,
    HailFilter_,
    HailMotto_,
    HailOpenCount_,
    HailTodos_,
    HailUserName_,
    PierTop_,
    PierUser_,
    TellAddTodo_,
    TellClearDone_,
    TellRemoveTodo_,
    TellToggleTodo_,
    Todo_id,
    Todo_status,
    Todo_text,
    asFilterAll,
    asFilterDone,
    asStatusDone,
} from "../../ahoi-wasm-tsain-todo/bindings/Tsain";

const isDone = (todo: Todo) => asStatusDone(Todo_status(todo)) !== undefined;

/**
 * Lives under the nested `User` pier: `Motto` is that pier's own state and
 * resets when the section unmounts, while `UserName` reaches the `Top`
 * state through the pier chain and survives.
 */
function Profile() {
    const pier = usePier();
    const [name, setName] = pier.hail(HailUserName_());
    const [motto, setMotto] = pier.hail(HailMotto_());

    return (
        <section>
            <label>
                name{" "}
                <input id="user-name" value={name()} onInput={(e) => setName(e.currentTarget.value)} />
            </label>{" "}
            <label>
                motto{" "}
                <input id="motto" value={motto()} onInput={(e) => setMotto(e.currentTarget.value)} />
            </label>
            <p id="greeting">
                ahoy, <b>{name()}</b> — “{motto()}”
            </p>
        </section>
    );
}

function NewTodo() {
    const pier = usePier();
    const [draft, setDraft] = createSignal("");

    const add = () => {
        const text = draft().trim();
        if (!text) return;
        pier.tell(TellAddTodo_(text));
        setDraft("");
    };

    return (
        <form
            onSubmit={(e) => {
                e.preventDefault();
                add();
            }}
        >
            <input
                id="new-todo"
                placeholder="What needs doing?"
                value={draft()}
                onInput={(e) => setDraft(e.currentTarget.value)}
            />
            <button id="add-todo" type="submit">
                add
            </button>
        </form>
    );
}

function TodoList() {
    const pier = usePier();
    const todos = pier.readHail(HailTodos_()); // () => Todo[]
    const openCount = pier.readHail(HailOpenCount_()); // () => number (memo)
    const [filter, setFilter] = pier.hail(HailFilter_()); // writable enum hail
    const [removed, setRemoved] = createSignal<Todo>();

    const visible = createMemo(() => {
        const f = filter();
        if (asFilterAll(f)) return todos();
        const wantDone = asFilterDone(f) !== undefined;
        return todos().filter((todo) => isDone(todo) === wantDone);
    });

    const FilterButton = (props: { label: string; value: Filter }) => (
        <button
            id={`filter-${props.label}`}
            classList={{ active: filter()[0] === props.value[0] }}
            onClick={() => setFilter(props.value)}
        >
            {props.label}
        </button>
    );

    const remove = (todo: Todo) => {
        const gone = pier.tell(TellRemoveTodo_(Todo_id(todo)));
        if (gone) setRemoved(gone);
    };

    return (
        <section>
            <p>
                <b id="open-count">{openCount()}</b> open ·{" "}
                <FilterButton label="all" value={FilterAll_()} />{" "}
                <FilterButton label="open" value={FilterOpen_()} />{" "}
                <FilterButton label="done" value={FilterDone_()} />{" "}
                <button id="clear-done" onClick={() => pier.tell(TellClearDone_())}>
                    clear done
                </button>
            </p>
            <ul id="todo-list">
                <For each={visible()}>
                    {(todo) => (
                        <li classList={{ done: isDone(todo) }}>
                            <input
                                type="checkbox"
                                checked={isDone(todo)}
                                onChange={() => pier.tell(TellToggleTodo_(Todo_id(todo)))}
                            />{" "}
                            <span>{Todo_text(todo)}</span>{" "}
                            <button onClick={() => remove(todo)}>✕</button>
                        </li>
                    )}
                </For>
            </ul>
            <Show when={removed()}>
                {(gone) => (
                    <p id="undo-bar">
                        deleted “{Todo_text(gone())}” —{" "}
                        <button
                            id="undo"
                            onClick={() => {
                                pier.tell(TellAddTodo_(Todo_text(gone())));
                                setRemoved(undefined);
                            }}
                        >
                            undo
                        </button>
                    </p>
                )}
            </Show>
        </section>
    );
}

export function App() {
    const [showProfile, setShowProfile] = createSignal(true);

    return (
        <PierProvider pier={PierTop_()}>
            <h1>ahoi × solid × tsain — todos</h1>
            <button id="toggle-profile" onClick={() => setShowProfile(!showProfile())}>
                {showProfile() ? "hide" : "show"} profile
            </button>
            <Show when={showProfile()}>
                <PierProvider pier={PierUser_()}>
                    <Profile />
                </PierProvider>
            </Show>
            <NewTodo />
            <TodoList />
        </PierProvider>
    );
}
