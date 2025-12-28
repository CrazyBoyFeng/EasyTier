package com.plugin.vpnservice

import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.os.Bundle
import java.net.InetAddress
import java.util.Arrays

import app.tauri.plugin.JSObject

class TauriVpnService : VpnService() {
    companion object {
        @JvmField var triggerCallback: (String, JSObject) -> Unit = { _, _ -> }
        @JvmField var self: TauriVpnService? = null

        const val IPV4_ADDR = "IPV4_ADDR"
        const val ROUTES = "ROUTES"
        const val DNS = "DNS"
        const val DISALLOWED_APPLICATIONS = "DISALLOWED_APPLICATIONS"
        const val MTU = "MTU"
        

        
        // Store the service instance for socket protection
        @JvmField var instance: TauriVpnService? = null
    }

    private lateinit var vpnInterface: ParcelFileDescriptor

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        println("vpn on start command ${intent?.getExtras()} $intent")
        var args = intent?.getExtras()

        vpnInterface = createVpnInterface(args)
        println("vpn created ${vpnInterface.fd}")

        var event_data = JSObject()
        event_data.put("fd", vpnInterface.fd)
        triggerCallback("vpn_service_start", event_data)

        return START_STICKY
    }

    override fun onCreate() {
        super.onCreate()
        self = this
        instance = this
        println("vpn on create")
    }

    override fun onDestroy() {
        println("vpn on destroy")
        super.onDestroy()
        disconnect()
        self = null
        instance = null
    }

    override fun onRevoke() {
        println("vpn on revoke")
        super.onRevoke()
        disconnect()
        self = null
    }

    private fun disconnect() {
        if (self == this && this::vpnInterface.isInitialized) {
            triggerCallback("vpn_service_stop", JSObject())
            vpnInterface.close()
        }
    }

    private fun createVpnInterface(args: Bundle?): ParcelFileDescriptor {
        var builder = Builder()
                .setSession("TauriVpnService")
                .setBlocking(false)
        
        var mtu = args?.getInt(MTU) ?: 1500
        var ipv4Addr = args?.getString(IPV4_ADDR) ?: "10.126.126.1/24"
        var dns: String? = args?.getString(DNS)
        var routes = args?.getStringArray(ROUTES) ?: emptyArray()
        var disallowedApplications = args?.getStringArray(DISALLOWED_APPLICATIONS) ?: emptyArray()

        println("vpn create vpn interface. mtu: $mtu, ipv4Addr: $ipv4Addr, dns:" +
            "$dns, routes: ${java.util.Arrays.toString(routes)}," +
            "disallowedApplications:  ${java.util.Arrays.toString(disallowedApplications)}")

        val ipParts = ipv4Addr.split("/")
        if (ipParts.size != 2) throw IllegalArgumentException("Invalid IP addr string")
        builder.addAddress(ipParts[0], ipParts[1].toInt())
        builder.addAddress("fd00::1", 128)

        builder.setMtu(mtu)
        dns?.let { builder.addDnsServer(it) }

        for (route in routes) {
            val ipParts = route.split("/")
            if (ipParts.size != 2) throw IllegalArgumentException("Invalid route cidr string")
            builder.addRoute(ipParts[0], ipParts[1].toInt())
        }
        
        for (app in disallowedApplications) {
            builder.addDisallowedApplication(app)
        }

        return builder.also {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                it.setMetered(false)
            }
        }
        .establish()
        ?: throw IllegalStateException("Failed to init VpnService")
    }
    
    /**
     * Protect a socket from the VPN by calling VpnService.protect()
     * This function is called from Rust via JNI
     * Returns 0 on success, -1 on failure
     */
    fun protectSocket(fd: Int): Int {
        return try {
            val success = protect(fd)
            if (success) {
                println("Successfully protected socket fd: $fd")
                0
            } else {
                println("Failed to protect socket fd: $fd")
                -1
            }
        } catch (e: Exception) {
            println("Exception protecting socket fd: $fd, error: $e")
            -1
        }
    }
    
    /**
     * Get the current service instance for socket protection
     * This function is called from JNI
     */
    companion object {
        @JvmStatic
        fun getInstance(): TauriVpnService? {
            return instance
        }
        
        /**
         * Static method to protect a socket using the current service instance
         * This method is called from Rust via JNI
         * Returns 0 on success, -1 on failure
         */
        @JvmStatic
        fun protectSocketStatic(fd: Int): Int {
            val service = instance
            return if (service != null) {
                service.protectSocket(fd)
            } else {
                println("Service instance not available for socket protection")
                -1
            }
        }
        
        /**
         * Initialize socket protection from native Rust code
         * This method ensures the service instance is available
         */
        @JvmStatic
        fun initProtectionFromRust(): Boolean {
            val service = instance
            if (service != null) {
                println("Socket protection initialized from Rust, service available")
                return true
            } else {
                println("Service not available for socket protection initialization")
                return false
            }
        }
    }
}
