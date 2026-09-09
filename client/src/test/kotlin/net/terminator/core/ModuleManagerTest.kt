package net.terminator.core

import net.terminator.core.event.EventBus
import net.terminator.core.event.TerminatorEvent
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class TestModule : Module() {
    override val name: String = "Test"
    override val category: ModuleCategory = ModuleCategory.Hud
    var enableCount = 0
    var disableCount = 0

    override fun onEnable() {
        enableCount++
    }

    override fun onDisable() {
        disableCount++
    }
}

class TestEvent : TerminatorEvent

class ModuleManagerTest {
    @BeforeEach
    fun setUp() {
        EventBus.clear()
    }

    @Test
    fun `toggle calls onEnable and onDisable once`() {
        val module = TestModule()
        assertFalse(module.enabled)

        module.toggle()
        assertTrue(module.enabled)
        assertEquals(1, module.enableCount)

        module.toggle()
        assertFalse(module.enabled)
        assertEquals(1, module.disableCount)
    }

    @Test
    fun `register and lookup by name is case-insensitive`() {
        ModuleManager.register(TestModule())
        assertTrue(ModuleManager.byName("test") != null)
        assertTrue(ModuleManager.byName("TEST") != null)
        assertTrue(ModuleManager.byName("missing") == null)
    }

    @Test
    fun `toggling by name returns false for unknown module`() {
        assertFalse(ModuleManager.toggleByName("nope"))
    }

    @Test
    fun `event bus delivers event to subscriber`() {
        var received = false
        EventBus.subscribe<TestEvent> { received = true }
        EventBus.post(TestEvent())
        assertTrue(received)
    }
}
