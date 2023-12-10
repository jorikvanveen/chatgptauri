<script lang="ts">
    import { listen, type Event } from "@tauri-apps/api/event"
    import Main from "./Main.svelte";
    import SettingsSvg from "./assets/settings.svg?raw";
    import Settings from "./Settings.svelte";
    import { writable } from "svelte/store";
    import { onMount, setContext } from "svelte";
    import Page from "./page";
    import type { ChatMessage } from "./chat";
    import type { Writable } from "svelte/store";
    import { invoke } from "@tauri-apps/api/tauri";
    import type { Model } from "./settings";

    let page = writable(Page.Main);
    setContext("page", page);

    let messages: Writable<ChatMessage[]> = writable([]);
    setContext("messages", messages);

    let apiKey = writable("");
    let model: Writable<Model> = writable("gpt3");

    let isLocked = writable(false);

    setContext("apiKey", apiKey);
    setContext("model", model);
    setContext("isLocked", isLocked);

    function toSettings() {
        $page = Page.Settings;
    }

    onMount(async () => {
        let settings: Settings = await invoke("get_settings");
        $apiKey = settings.apiKey || "";
        $model = settings.model;

        const unlistenAddContent = await listen("add_message_content", (event: Event<string>) => {
            const lastMessage = $messages[$messages.length - 1];
            lastMessage.content += event.payload;
            $messages = $messages;
        })

        const unlistenLock = await listen("lock", (event: Event<boolean>) => {
            $isLocked = event.payload
        })

        const unlistenRefreshMessages = await listen("refresh_messages", (event: Event<ChatMessage[]>) => {
            console.log(event.payload)
            $messages = event.payload;
        })

		const unlistenCost = await listen("cost", (event: Event<number>) => {
    	    const lastMessage = $messages[$messages.length - 1];
    	    lastMessage.cost_dollars = event.payload;
    	    $messages = $messages;
		})

        return () => {
            unlistenAddContent();
            unlistenLock();
            unlistenRefreshMessages();
			unlistenCost();
        }
    });
</script>

<div class="container">
    {#if $page == Page.Main}
        <button on:click={toSettings} class="nav-button">
            {@html SettingsSvg}
        </button>
        <Main />
    {:else if $page == Page.Settings}
        <Settings />
    {/if}
</div>

<style>
    .container {
        width: 100%;
        height: 100%;
        padding: 2rem;
        box-sizing: border-box;
    }

    .nav-button {
        background: none;
        border: none;
        outline: none;
        position: fixed;
        right: 0;
        cursor: pointer;
        padding: 0.5rem;
        fill: var(--fg);
    }
</style>
