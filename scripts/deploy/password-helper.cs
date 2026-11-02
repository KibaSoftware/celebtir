// Compiled locally with Windows PowerShell/.NET Framework; no external package.
// SSH_ASKPASS writes only to SSH's private pipe. --run relays an interactive
// deployment and sends sudo's password only after remote terminal echo is off.
using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

public static class CelebtirPasswordHelper
{
    static string Password()
    {
        string hex = File.ReadAllText(Environment.GetEnvironmentVariable("CELEBTIR_PASSWORD_FILE")).Trim();
        byte[] encrypted = new byte[hex.Length / 2];
        for (int i = 0; i < encrypted.Length; i++) encrypted[i] = Convert.ToByte(hex.Substring(i * 2, 2), 16);
        byte[] plain = ProtectedData.Unprotect(encrypted, null, DataProtectionScope.CurrentUser);
        try { return Encoding.Unicode.GetString(plain); }
        finally { Array.Clear(plain, 0, plain.Length); }
    }

    // Windows CommandLineToArgvW quoting, including trailing backslashes.
    static string Quote(string value)
    {
        var result = new StringBuilder("\"");
        int slashes = 0;
        foreach (char c in value)
        {
            if (c == '\\') { slashes++; continue; }
            result.Append('\\', c == '"' ? slashes * 2 + 1 : slashes);
            result.Append(c); slashes = 0;
        }
        return result.Append('\\', slashes * 2).Append('"').ToString();
    }

    static async Task ForwardInput(Stream source, Stream destination)
    {
        // Flush every read, including short interactive responses on redirected
        // Windows console handles; CopyToAsync alone can leave them buffered.
        byte[] buffer = new byte[4096];
        int count;
        while ((count = await source.ReadAsync(buffer, 0, buffer.Length)) != 0)
        {
            await destination.WriteAsync(buffer, 0, count);
            await destination.FlushAsync();
        }
    }

    static int Relay(string[] args)
    {
        if (args.Length < 4) throw new ArgumentException("Invalid deployment relay arguments.");
        string marker = args[1];
        string command = Encoding.UTF8.GetString(Convert.FromBase64String(args[2]));
        string[] sshArgs = args.Skip(4).Concat(new[] { command }).ToArray();
        var start = new ProcessStartInfo(args[3], String.Join(" ", sshArgs.Select(Quote))) {
            UseShellExecute = false, CreateNoWindow = true,
            RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true,
            StandardOutputEncoding = new UTF8Encoding(false), StandardErrorEncoding = new UTF8Encoding(false)
        };
        using (var process = Process.Start(start))
        using (var input = new StreamWriter(process.StandardInput.BaseStream, new UTF8Encoding(false)) { AutoFlush = true, NewLine = "\n" })
        {
            char[] stdout = new char[4096], stderr = new char[4096];
            Task<int> outputRead = process.StandardOutput.ReadAsync(stdout, 0, stdout.Length);
            Task<int> errorRead = process.StandardError.ReadAsync(stderr, 0, stderr.Length);
            var pending = new StringBuilder();
            var timer = Stopwatch.StartNew();
            bool sent = false;
            bool console = !Console.IsInputRedirected;
            bool oldControlC = console && Console.TreatControlCAsInput;
            Task redirectedInput = null;
            try
            {
                if (console) Console.TreatControlCAsInput = true;
                while (outputRead != null || errorRead != null || !process.HasExited)
                {
                    if (!sent && timer.Elapsed.TotalSeconds > 120) throw new TimeoutException("Timed out waiting for the remote password channel.");
                    if (outputRead != null && outputRead.IsCompleted)
                    {
                        int count = outputRead.GetAwaiter().GetResult();
                        if (count == 0) outputRead = null;
                        else
                        {
                            if (sent) Console.Write(stdout, 0, count);
                            else
                            {
                                pending.Append(stdout, 0, count);
                                int position = pending.ToString().IndexOf(marker, StringComparison.Ordinal);
                                if (position >= 0)
                                {
                                    Console.Write(pending.ToString(0, position));
                                    Console.Write(pending.ToString(position + marker.Length, pending.Length - position - marker.Length));
                                    pending.Clear();
                                    // The remote shell consumes exactly one frame even with NOPASSWD sudo.
                                    input.WriteLine(Password());
                                    sent = true;
                                    if (!console) redirectedInput = ForwardInput(Console.OpenStandardInput(), input.BaseStream);
                                }
                                else if (pending.Length > marker.Length)
                                {
                                    int flush = pending.Length - marker.Length;
                                    Console.Write(pending.ToString(0, flush)); pending.Remove(0, flush);
                                }
                            }
                            outputRead = process.StandardOutput.ReadAsync(stdout, 0, stdout.Length);
                        }
                    }
                    if (errorRead != null && errorRead.IsCompleted)
                    {
                        int count = errorRead.GetAwaiter().GetResult();
                        if (count == 0) errorRead = null;
                        else { Console.Error.Write(stderr, 0, count); errorRead = process.StandardError.ReadAsync(stderr, 0, stderr.Length); }
                    }
                    if (sent && console && !process.HasExited && Console.KeyAvailable)
                    {
                        ConsoleKeyInfo key = Console.ReadKey(true);
                        if (key.Key == ConsoleKey.Backspace) input.Write('\x7f');
                        else if (key.KeyChar != '\0') input.Write(key.KeyChar);
                    }
                    if (redirectedInput != null && redirectedInput.IsFaulted) redirectedInput.GetAwaiter().GetResult();
                    Thread.Sleep(10);
                }
                Console.Write(pending.ToString());
                process.WaitForExit();
                if (!sent && process.ExitCode == 0) throw new InvalidOperationException("Remote password handshake was not received.");
                return process.ExitCode;
            }
            finally
            {
                if (console) Console.TreatControlCAsInput = oldControlC;
                if (!process.HasExited) { process.Kill(); process.WaitForExit(); }
            }
        }
    }

    public static int Main(string[] args)
    {
        Console.OutputEncoding = new UTF8Encoding(false);
        try
        {
            if (args.Length > 0 && args[0] == "--run") return Relay(args);
            // Never answer host-key confirmation, a private-key passphrase, or MFA with this password.
            if (args.Length != 1 || !args[0].TrimEnd().EndsWith("password:", StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("Only the SSH account password can be reused. Unlock private keys in ssh-agent separately.");
            Console.WriteLine(Password());
            return 0;
        }
        catch (Exception)
        {
            // Do not dump arguments, child input, credential paths, or exception data.
            Console.Error.WriteLine("Celebtir password transport failed. Check SSH authentication/known_hosts and use the same account password for sudo.");
            return 1;
        }
    }
}
