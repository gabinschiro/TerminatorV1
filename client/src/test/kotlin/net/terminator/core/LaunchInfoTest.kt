package net.terminator.core

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class LaunchInfoTest {
    @Test
    fun `parses terminator args`() {
        val args =
            arrayOf(
                "--terminator-version", "0.1.0",
                "--terminator-dir", "/home/user/.terminator",
                "--terminator-username", "zqodev",
                "--terminator-uuid", "f0dac655edd143a3ac51c07980a94244",
                "--terminator-access-token", "tok",
            )
        val info = LaunchInfo.fromArgs(args)
        assertNotNull(info)
        assertEquals("0.1.0", info!!.version)
        assertEquals("/home/user/.terminator", info.gameDir)
        assertEquals("zqodev", info.username)
        assertEquals("tok", info.accessToken)
    }

    @Test
    fun `returns null when a required arg is missing`() {
        assertNull(LaunchInfo.fromArgs(arrayOf("--terminator-version", "0.1.0")))
        assertNull(LaunchInfo.fromArgs(emptyArray()))
    }
}
