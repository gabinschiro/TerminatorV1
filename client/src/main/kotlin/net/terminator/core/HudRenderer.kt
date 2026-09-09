package net.terminator.core

import net.minecraft.client.gui.DrawContext

interface HudRenderer {
    fun render(
        context: DrawContext,
        tickDelta: Float,
    )
}
