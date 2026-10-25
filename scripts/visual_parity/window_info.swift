import AppKit
import CoreGraphics

// Read only the selected window's identity and geometry, never other apps' titles.
guard CommandLine.arguments.count == 2, let windowID = UInt32(CommandLine.arguments[1]) else {
    fputs("Expected window ID\n", stderr)
    exit(2)
}
let session = CGSessionCopyCurrentDictionary() as? [String: Any]
guard session?["CGSSessionScreenIsLocked"] as? Bool != true else {
    fputs("Screen is locked\n", stderr)
    exit(2)
}
guard let windows = CGWindowListCopyWindowInfo(.optionIncludingWindow, windowID) as? [[String: Any]],
      let window = windows.first(where: { ($0[kCGWindowNumber as String] as? UInt32) == windowID }),
      let pid = window[kCGWindowOwnerPID as String] as? Int32,
      let bounds = window[kCGWindowBounds as String] as? [String: Any],
      let app = NSRunningApplication(processIdentifier: pid),
      let executable = app.executableURL else {
    fputs("Window or process identity unavailable\n", stderr)
    exit(2)
}
let metadata: [String: Any] = [
    "window_id": windowID, "pid": pid, "bounds": bounds,
    "bundle_path": app.bundleURL?.path ?? "",
    "bundle_id": app.bundleIdentifier ?? "",
    "executable": executable.path,
    "visible": window[kCGWindowIsOnscreen as String] as? Bool ?? false,
    "platform": ProcessInfo.processInfo.operatingSystemVersionString
]
let output = try JSONSerialization.data(withJSONObject: metadata, options: [.sortedKeys])
print(String(decoding: output, as: UTF8.self))
