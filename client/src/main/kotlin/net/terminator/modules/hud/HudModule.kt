package net.terminator.modules.hud

import net.minecraft.client.MinecraftClient
import net.minecraft.client.gui.DrawContext
import net.minecraft.text.Text
import net.terminator.core.HudRenderer
import net.terminator.core.Module
import net.terminator.core.ModuleCategory

object HudModule : Module(), HudRenderer {
    override val name: String = "HUD"
    override val category: ModuleCategory = ModuleCategory.Hud
    override val description: String = "Affiche FPS, coordonnées et direction du joueur."

    override fun render(
        context: DrawContext,
        tickDelta: Float,
    ) {
        val client = MinecraftClient.getInstance()
        val player = client.player
        if (player == null || client.world == null) return

        val textRenderer = client.textRenderer
        val fps = client.getCurrentFps()

        val coords =
            String.format(
                "%.0f %.0f %.0f",
                player.x,
                player.y,
                player.z,
            )
        val lines = listOf("Terminator V1  |  $fps FPS", coords)

        var y = 4
        lines.forEach { line ->
            context.drawText(textRenderer, Text.literal(line), 4, y, 0xFFFFFF, true)
            y += textRenderer.fontHeight
        }
    }
}
