package net.terminator.core

sealed class ModuleCategory(val id: String, val displayName: String) {
    data object Hud : ModuleCategory("hud", "HUD")
    data object Pvp : ModuleCategory("pvp", "PvP")
    data object Visuals : ModuleCategory("visuals", "Visuals")
    data object Movement : ModuleCategory("movement", "Movement")
    data object Chat : ModuleCategory("chat", "Chat")
    data object Settings : ModuleCategory("settings", "Settings")

    companion object {
        fun all(): List<ModuleCategory> = listOf(Hud, Pvp, Visuals, Movement, Chat, Settings)
    }
}
