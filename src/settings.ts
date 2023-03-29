export type Model = "gpt3" | "gpt4" | "gpt432k";

export interface Settings {
    openai_key: string | null;
    model: Model;
}

