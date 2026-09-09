package net.terminator.core.event

import net.minecraft.client.MinecraftClient
import net.minecraft.client.gui.DrawContext

class ClientTickEvent : TerminatorEvent {
    val client: MinecraftClient get() = MinecraftClient.getInstance()
}

class HudRenderEvent(
    val context: DrawContext,
    val tickDelta: Float,
) : TerminatorEvent
