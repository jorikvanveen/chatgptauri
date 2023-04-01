<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { getContext, tick } from "svelte";
    import MultilineParagraph from "./MultilineParagraph.svelte";
    import { marked } from "marked";
    import type { ChatMessage } from "./chat";
    import type { Writable } from "svelte/store";
    import type { Model } from "./settings";
    import renderLatex from "./renderlatex";

    interface PromptResponse {
        cost: number;
        content: string;
    }

    let promptInput = "";
    let messages: Writable<ChatMessage[]> = getContext("messages");

    function promptKeyDown(e: KeyboardEvent) {
        if (e.key == "Enter" && !e.shiftKey) {
            e.preventDefault();
            submitPrompt(promptInput);
            promptInput = "";
        }
    }

    async function submitPrompt(prompt: string) {
        let chatlog = document.querySelector(".chatlog");

        $messages.push({ role: "user", message: prompt, cost: null })
        messages = messages
        await tick()

        chatlog.scrollTo(0, chatlog.scrollHeight)

        try {
            console.log("Requesting")
            let resp = await invoke("prompt", { prompt }) as PromptResponse;
            $messages.push({role: "assistant", message: resp.content, cost: resp.cost})
            console.log("Success")
        } catch (e) {
            $messages.push({role: "error", message: e.toString(), cost: null});
            console.error(e);
        }

        messages = messages;
        await tick()
        chatlog.scrollTo(0, chatlog.scrollHeight)
    }
</script>

<main class="container">
    <div class="chatlog">
        {#each $messages as message}
            {#if message.role == "user"}
                <p class="msg user"><MultilineParagraph text={message.message} /></p>
            {:else if message.role == "assistant"}
                <p class="msg assistant">{@html marked.parse(renderLatex(message.message))}(<span class="cost">${message.cost.toPrecision(3)}</span>)</p>
            {:else}
                <p class="msg error"><MultilineParagraph text={message.message} /></p>
            {/if}
        {/each}
    </div>
    <div class="promptarea">
        <textarea bind:value={promptInput} on:keydown={promptKeyDown} />
    </div>
</main>

<style>
    .container {
        width: 100%;
        height: 100%;
        box-sizing: border-box; 
        display: flex;
        flex-direction: column;
    }

    .chatlog {
        height: 80%;
        overflow-y: scroll;
    }

    .promptarea {
        height: 20%;
    }

    .user {
        font-weight: bold;
        padding-left: 2rem;
        border-left: solid 3px var(--teal);
    }

    .error {
        color: var(--light-red);
        border-color: var(--dark-red);
    }

    .cost {
        color: var(--teal);
    }

    textarea {
        resize: none;
        outline: none;
        width: 100%;
        height: 100%;
        border-radius: 8px;
        background: var(--dark-blue);
        padding: 1rem;
        box-sizing: border-box;
        color: var(--fg);
        border: none;
    }
</style>
