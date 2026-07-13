import Foundation

@MainActor
final class ModelDownloader: NSObject, ObservableObject {
    @Published var progress: Double = 0
    @Published var isDownloading = false
    @Published var error: String?

    /// ggml-small.bin is ~466 MB; anything smaller is an error page or a
    /// truncated body, even if the HTTP layer called it a success.
    nonisolated static let minimumValidSize: Int64 = 400_000_000

    private var session: URLSession?
    private var task: URLSessionDownloadTask?

    func start() {
        guard !isDownloading else { return }
        error = nil
        progress = 0
        isDownloading = true
        try? FileManager.default.createDirectory(
            at: WhisperEngine.modelsDirectory, withIntermediateDirectories: true)
        let session = URLSession(configuration: .default, delegate: self, delegateQueue: nil)
        self.session = session
        task = session.downloadTask(with: WhisperEngine.modelURL)
        task?.resume()
    }

    func cancel() {
        task?.cancel()
        task = nil
        isDownloading = false
        tearDownSession()
    }

    /// URLSession retains its delegate until invalidated; without this the
    /// downloader (and the session's queue) leak for the app's lifetime.
    private func tearDownSession() {
        session?.finishTasksAndInvalidate()
        session = nil
    }

    private func finish(errorMessage: String?) {
        isDownloading = false
        progress = errorMessage == nil ? 1 : 0
        error = errorMessage
        task = nil
        tearDownSession()
    }
}

extension ModelDownloader: URLSessionDownloadDelegate {
    nonisolated func urlSession(
        _ session: URLSession, downloadTask: URLSessionDownloadTask,
        didWriteData bytesWritten: Int64, totalBytesWritten: Int64,
        totalBytesExpectedToWrite: Int64
    ) {
        guard totalBytesExpectedToWrite > 0 else { return }
        let fraction = Double(totalBytesWritten) / Double(totalBytesExpectedToWrite)
        Task { @MainActor in self.progress = fraction }
    }

    nonisolated func urlSession(
        _ session: URLSession, downloadTask: URLSessionDownloadTask,
        didFinishDownloadingTo location: URL
    ) {
        // Validate before moving: a 404/500 body or captive-portal page also
        // lands here "successfully". Work synchronously — `location` is
        // deleted when this method returns.
        var failure: String?
        let status = (downloadTask.response as? HTTPURLResponse)?.statusCode ?? 0
        let attrs = try? FileManager.default.attributesOfItem(atPath: location.path)
        let size = (attrs?[.size] as? Int64) ?? 0

        if status != 200 {
            failure = "Download failed (HTTP \(status)). Try again."
        } else if size < Self.minimumValidSize {
            failure = "Download incomplete (\(size / 1_000_000) MB of ~466 MB). "
                + "Check your connection and try again."
        } else {
            do {
                let dest = WhisperEngine.modelPath
                try? FileManager.default.removeItem(at: dest)
                try FileManager.default.moveItem(at: location, to: dest)
            } catch {
                failure = error.localizedDescription
            }
        }

        let message = failure
        Task { @MainActor in self.finish(errorMessage: message) }
    }

    nonisolated func urlSession(
        _ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?
    ) {
        guard let error, (error as NSError).code != NSURLErrorCancelled else { return }
        let message = error.localizedDescription
        Task { @MainActor in self.finish(errorMessage: message) }
    }
}
