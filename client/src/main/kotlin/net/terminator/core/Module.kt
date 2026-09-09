package net.terminator.core

abstract class Module {
    abstract val name: String
    abstract val category: ModuleCategory
    var keyBind: Int = 0

    open val description: String = ""

    private var _enabled: Boolean = false
    var enabled: Boolean
        get() = _enabled
        set(value) {
            if (_enabled == value) return
            _enabled = value
            if (value) onEnable() else onDisable()
        }

    fun toggle() {
        enabled = !enabled
    }

    open fun onEnable() {}

    open fun onDisable() {}

    override fun toString(): String = name
}
