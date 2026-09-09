package net.terminator

import net.fabricmc.api.ClientModInitializer
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback
import net.minecraft.client.option.KeyBinding
import net.minecraft.client.util.InputUtil
import net.terminator.core.HudRenderer
import net.terminator.core.ModuleManager
import net.terminator.core.event.ClientTickEvent
import net.terminator.core.event.EventBus
import net.terminator.core.event.HudRenderEvent
import net.terminator.modules.hud.HudModule
import org.lwjgl.glfw.GLFW

object TerminatorClient : ClientModInitializer {
    private val hudKeyBinding: KeyBinding =
        KeyBinding(
            "key.terminator.hud",
            InputUtil.Type.KEYSYM,
            GLFW.GLFW_KEY_H,
            "category.terminator",
        )

    override fun onInitializeClient() {
        ModuleManager.register(HudModule)
        KeyBindingHelper.registerKeyBinding(hudKeyBinding)

        ClientTickEvents.END_CLIENT_TICK.register { client ->
            while (hudKeyBinding.wasPressed()) {
                HudModule.toggle()
            }
            EventBus.post(ClientTickEvent())
        }

        HudRenderCallback.EVENT.register { drawContext, tickCounter ->
            val tickDelta = tickCounter.getTickDelta(false)
            EventBus.post(HudRenderEvent(drawContext, tickDelta))
            ModuleManager.all()
                .filter { it.enabled && it is HudRenderer }
                .forEach { (it as HudRenderer).render(drawContext, tickDelta) }
        }

        ModuleManager.loadConfig()
        Runtime.getRuntime().addShutdownHook(Thread { ModuleManager.saveConfig() })
    }
}
