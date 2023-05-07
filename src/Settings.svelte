<script lang="ts" context="module">
    interface Conversation {
        name: string;
        id: number;
        messages: ChatMessage[];
    }
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import Input from "./lib/Input.svelte";
    import Button from "./lib/Button.svelte";
    import type { ChatMessage } from "./chat";
    import type { Writable } from "svelte/store";
    import { getContext, onMount } from "svelte";
    import type { Settings } from "./settings";
    import Page from "./page";
    import CloseSvg from "./assets/close.svg?raw";

    let apiKey: Writable<string> = getContext("apiKey");
    let model: "gpt3" | "gpt4" | "gpt432k" = getContext("model");

    let messages: Writable<ChatMessage[]> = getContext("messages");
    let isLocked: Writable<boolean> = getContext("isLocked");
    let page: Writable<Page> = getContext("page");

    let conversations: Conversation[] = [];
    let conversation_id: number = -1;

    async function clearMessages() {
        $messages = [];
        await invoke("clear_messages");
        toMain();
    }

    async function getSettings(): Promise<Settings> {
        return await invoke("get_settings");
    }

    async function updateSettings() {
        let settings = { openai_key: $apiKey || null, model };
        await invoke("update_settings", { settingsNew: settings });
    }

    async function toMain() {
        await updateSettings();
        $page = Page.Main;
    }

    async function loadConversation(id: number) {
        if ($isLocked) {
            return
        }
        console.log("Loading", id);
        await invoke("load_conversation", { newConversationId: id })
        toMain();
    }

    async function newConversation() {
        await invoke("reset_conversation");
        toMain();
    }

    onMount(async () => {
        let settings = await getSettings();
        $apiKey = settings.openai_key || "";
        model = settings.model;

        conversations = await invoke("list_conversations");
        console.log(conversations);
        conversation_id = await invoke("get_current_conversation_id");
    });
</script>

<h1>Settings</h1>

<button on:click={toMain} class="nav-button">
    {@html CloseSvg}
</button>
<div class="container">
    <Button
        disabled={$isLocked}
        on:click={clearMessages}
        label="Clear messages"
    /><br />
    <Input label="API Key" password bind:value={$apiKey} />
    <br />

    <label for="model">Model</label>
    <select bind:value={model} id="model">
        <option value="gpt3">GPT 3</option>
        <option value="gpt4">GPT 4</option>
        <option value="gpt432k">GPT 4 32K</option>
    </select>
    <br />
    <h2>Conversations</h2>
    <Button label="New conversation" on:click={newConversation} />
    <div>
        {#each conversations as conversation}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <p
                class={conversation_id == conversation.id
                    ? "current convo"
                    : "convo"}
                on:click={loadConversation.bind(this, conversation.id)}
            >
                {conversation.name}
            </p>
        {/each}
    </div>
</div>

<style>
    .container {
        display: flex;
        flex-direction: column;
        max-width: 50rem;
    }

    .nav-button {
        background: none;
        border: none;
        outline: none;
        position: fixed;
        right: 0;
        top: 0;
        cursor: pointer;
        padding: 0.5rem;
        fill: var(--fg);
    }

    .convo {
        border: solid 2px var(--less-dark-blue);
        border-radius: 0.5rem;
        padding: 0.7rem;
        cursor: pointer;
    }

    .current {
        border: solid 2px var(--teal);
    }
</style>
