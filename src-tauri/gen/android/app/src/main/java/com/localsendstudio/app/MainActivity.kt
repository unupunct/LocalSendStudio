package com.localsendstudio.app

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.Settings
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  private var multicastLock: WifiManager.MulticastLock? = null

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    // Android drops multicast/broadcast packets while the screen is on unless an app holds this lock.
    // Without it other devices can't be discovered.
    try {
      val wifi = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
      multicastLock = wifi.createMulticastLock("localsendstudio").apply {
        setReferenceCounted(false)
        acquire()
      }
    } catch (_: Exception) {}

    // Needed to read files you pick and to save received files in Downloads/LocalSendStudio.
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R && !Environment.isExternalStorageManager()) {
      try {
        startActivity(Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:$packageName")))
      } catch (_: Exception) {}
    }
  }

  override fun onDestroy() {
    try { multicastLock?.release() } catch (_: Exception) {}
    super.onDestroy()
  }
}
