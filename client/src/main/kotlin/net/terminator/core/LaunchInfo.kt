package net.terminator.core

data class LaunchInfo(
    val version: String,
    val gameDir: String,
    val username: String,
    val uuid: String,
    val accessToken: String,
) {
    companion object {
        private const val PREFIX = "--terminator-"

        fun fromArgs(args: Array<String>): LaunchInfo? {
            val values = mutableMapOf<String, String>()
            var i = 0
            while (i < args.size) {
                val arg = args[i]
                if (arg.startsWith(PREFIX) && i + 1 < args.size) {
                    values[arg.removePrefix(PREFIX)] = args[i + 1]
                    i += 2
                } else {
                    i += 1
                }
            }

            val version = values["version"] ?: return null
            val gameDir = values["dir"] ?: return null
            val username = values["username"] ?: return null
            val uuid = values["uuid"] ?: return null
            val accessToken = values["access-token"] ?: return null

            return LaunchInfo(version, gameDir, username, uuid, accessToken)
        }

        // La JVM expose la ligne de commande complète dans sun.java.command :
        // les args --terminator-* passés par le launcher y sont lisibles sans mixin.
        fun current(): LaunchInfo? {
            val command = System.getProperty("sun.java.command") ?: return null
            return fromArgs(command.split(" ").toTypedArray())
        }
    }
}
