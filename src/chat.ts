export interface ChatMessage {
    role: "user" | "assistant" | "error"
    message: string,
    cost: number | null    
}
