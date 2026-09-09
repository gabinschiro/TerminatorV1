package net.terminator.mixins

import net.minecraft.client.MinecraftClient
import net.minecraft.client.session.Session
import net.terminator.core.LaunchInfo
import org.spongepowered.asm.mixin.Mixin
import org.spongepowered.asm.mixin.injection.At
import org.spongepowered.asm.mixin.injection.Inject
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable
import java.util.Optional
import java.util.UUID

@Mixin(MinecraftClient::class)
class MinecraftClientMixin {
    @Inject(method = ["getSession"], at = [At("HEAD")], cancellable = true)
    fun terminatorInjectSession(cir: CallbackInfoReturnable<Session>) {
        val launch = LaunchInfo.current() ?: return
        val session =
            Session(
                launch.username,
                UUID.fromString(launch.uuid),
                launch.accessToken,
                Optional.empty(),
                Optional.empty(),
                Session.AccountType.MSA,
            )
        cir.returnValue = session
    }
}
