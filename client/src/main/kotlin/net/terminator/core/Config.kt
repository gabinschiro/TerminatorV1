package net.terminator.core

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import net.fabricmc.loader.api.FabricLoader
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path

@Serializable
data class ModuleConfigData(
    val name: String,
    val enabled: Boolean = false,
    val keyBind: Int = 0,
)

@Serializable
data class TerminatorConfigData(
    val modules: List<ModuleConfigData> = emptyList(),
)

object Config {
    private val json =
        Json {
            prettyPrint = true
            encodeDefaults = true
            ignoreUnknownKeys = true
        }

    val configPath: Path =
        FabricLoader.getInstance().gameDir.resolve("config").resolve("terminator.json")

    fun load(): TerminatorConfigData {
        return try {
            if (!Files.exists(configPath)) return TerminatorConfigData()
            val content = Files.readString(configPath)
            json.decodeFromString<TerminatorConfigData>(content)
        } catch (e: IOException) {
            TerminatorConfigData()
        } catch (e: Exception) {
            TerminatorConfigData()
        }
    }

    fun save(data: TerminatorConfigData) {
        try {
            configPath.parent?.let { Files.createDirectories(it) }
            Files.writeString(configPath, json.encodeToString(data))
        } catch (e: IOException) {
            // non-blocking : la config est sauvée au shutdown, ignorer un échec ponctuel
        }
    }
}
