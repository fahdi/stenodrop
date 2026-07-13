import AppKit
import SwiftUI

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Needed when running as a bare SwiftPM executable (no bundle LSUIElement handling).
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        if JobQueue.shared.hasActiveWork {
            let alert = NSAlert()
            alert.messageText = "Transcription in progress"
            alert.informativeText =
                "Files still in the queue won't be transcribed if you quit now."
            alert.addButton(withTitle: "Quit Anyway")
            alert.addButton(withTitle: "Keep Transcribing")
            if alert.runModal() != .alertFirstButtonReturn {
                return .terminateCancel
            }
        }
        // Never leave an orphaned ffmpeg/whisper child burning CPU.
        WhisperEngine.terminateActiveProcesses()
        return .terminateNow
    }
}

@main
struct VibeTranscribeApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @StateObject private var queue = JobQueue.shared

    var body: some Scene {
        WindowGroup("VibeTranscribe") {
            RootView()
                .environmentObject(queue)
                .frame(minWidth: 560, minHeight: 480)
        }
    }
}
