<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { getContext, onMount, tick } from "svelte";
    import MultilineParagraph from "./MultilineParagraph.svelte";
    import { marked } from "marked";
    import type { ChatMessage } from "./chat";
    import type { Writable } from "svelte/store";
    import renderLatex from "./renderlatex";

    let isLocked: Writable<boolean> = getContext("isLocked");
    let messages: Writable<ChatMessage[]> = getContext("messages");
    let promptInput = "";

    function promptKeyDown(e: KeyboardEvent) {
        if (e.key == "Enter" && !e.shiftKey) {
            e.preventDefault();
            submitPrompt(promptInput);
            promptInput = "";
        }
    }

    // Scroll down if the messages update
    $: {
        $messages;
        const chatLog = document.querySelector(".chatlog");
        if (chatLog) {
            scrollDown();
        }
    }

    function scrollDown() {
        let chatlog = document.querySelector(".chatlog");
        chatlog.scrollTo(0, chatlog.scrollHeight);
    }
 
    async function submitPrompt(prompt: string) {
        $messages.push({ role: "user", content: prompt });
        messages = messages;
        await tick();

        scrollDown()

        try {
            console.log("Requesting");
            await invoke("prompt", { prompt });
            $messages.push({ role: "assistant", content: ""});
            console.log("Success");
        } catch (e) {
            $messages.push({
                role: "error",
                content: e.toString(),
            });
            console.error(e);
        }

        messages = messages;
        await tick();
        scrollDown();
    }

    onMount(() => {
        scrollDown();
    })
</script>

<main class="container">
    <div class="chatlog">
        {#each $messages as message}
            {#if message.role == "user"}
                <p class="msg user">
                    <MultilineParagraph text={message.content} />
                </p>
            {:else if message.role == "assistant"}
                <p class="msg assistant">
                    {@html marked.parse(renderLatex(message.content))}
                    <!-- (<span class="cost">${message.cost.toPrecision(3)}</span>) -->
                </p>
				{#if message.cost}
					<span>{message.cost}</span>
				{/if}
            {:else}
                <p class="msg error">
                    <MultilineParagraph text={message.content} />
                </p>
            {/if}
        {/each}
    </div>
    <div class="promptarea">
        <!-- svelte-ignore a11y-autofocus -->
        <textarea autofocus disabled={$isLocked} bind:value={promptInput} on:keydown={promptKeyDown} />
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

    textarea:disabled {
        opacity: 0.5;
    }
</style>
