package net.terminator.core

import java.util.concurrent.CopyOnWriteArrayList

object ModuleManager {
    private val modules = CopyOnWriteArrayList<Module>()

    fun register(module: Module) {
        modules.add(module)
    }

    fun all(): List<Module> = modules

    fun byName(name: String): Module? = modules.find { it.name.equals(name, ignoreCase = true) }

    fun byCategory(category: ModuleCategory): List<Module> = modules.filter { it.category == category }

    fun toggleByName(name: String): Boolean {
        val module = byName(name) ?: return false
        module.toggle()
        return true
    }

    fun onKeyPress(key: Int) {
        modules.firstOrNull { it.keyBind == key }?.toggle()
    }

    fun loadConfig() {
        val data = Config.load()
        data.modules.forEach { saved ->
            val module = byName(saved.name) ?: return@forEach
            module.enabled = saved.enabled
            module.keyBind = saved.keyBind
        }
    }

    fun saveConfig() {
        val data =
            TerminatorConfigData(
                modules =
                    modules.map {
                        ModuleConfigData(it.name, it.enabled, it.keyBind)
                    },
            )
        Config.save(data)
    }
}
