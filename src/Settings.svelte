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

    async function clearMessages() {
        $messages = [];
        await invoke("clear_messages");
        console.log("cleared");
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

    onMount(async () => {
        let settings = await getSettings();
        $apiKey = settings.openai_key || "";
        model = settings.model;
    });
</script>

<h1>Settings</h1>

<button on:click={toMain} class="nav-button">
    {@html CloseSvg}
</button>
<div class="container">
    <Button disabled={$isLocked} on:click={clearMessages} label="Clear messages" /><br />
    <Input label="API Key" password bind:value={$apiKey} />
    <br />

    <label for="model">Model</label>
    <select bind:value={model} id="model">
        <option value="gpt3">GPT 3</option>
        <option value="gpt4">GPT 4</option>
        <option value="gpt432k">GPT 4 32K</option>
    </select>
    <br />
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
</style>
