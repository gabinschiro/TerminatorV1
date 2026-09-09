package net.terminator.core.event

import java.util.concurrent.ConcurrentHashMap

object EventBus {
    @PublishedApi
    internal val listeners: MutableMap<Class<out TerminatorEvent>, MutableList<(TerminatorEvent) -> Unit>> =
        ConcurrentHashMap()

    inline fun <reified T : TerminatorEvent> subscribe(noinline handler: (T) -> Unit) {
        val type = T::class.java
        val list = listeners.getOrPut(type) { mutableListOf() }
        list.add { event ->
            @Suppress("UNCHECKED_CAST")
            handler(event as T)
        }
    }

    inline fun <reified T : TerminatorEvent> post(event: T) {
        val type = T::class.java
        listeners[type]?.forEach { handler ->
            @Suppress("UNCHECKED_CAST")
            (handler as (T) -> Unit)(event)
        }
    }

    fun clear() {
        listeners.clear()
    }
}
