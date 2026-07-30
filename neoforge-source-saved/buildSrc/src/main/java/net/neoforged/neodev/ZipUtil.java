package net.neoforged.neodev;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.Calendar;
import java.util.List;
import java.util.zip.CRC32;
import java.util.zip.Deflater;
import java.util.zip.DeflaterOutputStream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

public class ZipUtil {

    public static void stripNtfsExtra(Path jarPath) throws IOException {
        var tmp = jarPath.resolveSibling("." + jarPath.getFileName() + ".ntfs-stripped");
        try {
            copyZipStrippingNtfsExtra(jarPath, tmp);
            Files.move(tmp, jarPath, StandardCopyOption.REPLACE_EXISTING);
        } catch (Exception e) {
            try { Files.deleteIfExists(tmp); } catch (Exception ignored) {}
            throw new IOException("Failed to strip NTFS extra data from " + jarPath, e);
        }
    }

    public static void copyZipStrippingNtfsExtra(Path source, Path target) throws IOException {
        record EntryData(String name, int method, byte[] data, long time, long crc) {}

        List<EntryData> entries = new ArrayList<>();
        try (var zf = new ZipFile(source.toFile())) {
            var zfe = zf.entries();
            byte[] buf = new byte[8192];
            while (zfe.hasMoreElements()) {
                var ze = zfe.nextElement();
                try (var is = zf.getInputStream(ze)) {
                    var bos = new ByteArrayOutputStream((int) Math.max(ze.getSize(), 4096));
                    int r;
                    while ((r = is.read(buf)) > 0) bos.write(buf, 0, r);
                    entries.add(new EntryData(ze.getName(), ze.getMethod(), bos.toByteArray(), ze.getTime(), ze.getCrc()));
                }
            }
        }

        try (var out = Files.newOutputStream(target); var cd = new ByteArrayOutputStream()) {
            long localOffset = 0;
            for (var e : entries) {
                byte[] dataToWrite;
                long compSize;
                int method;
                long crcVal;

                if (e.data.length == 0) {
                    dataToWrite = e.data; compSize = 0; method = 0; crcVal = e.crc;
                } else if (e.method == 0) {
                    dataToWrite = e.data; compSize = e.data.length; method = 0; crcVal = e.crc;
                } else {
                    var comp = new ByteArrayOutputStream(e.data.length / 2);
                    var deflater = new Deflater(Deflater.DEFAULT_COMPRESSION, true);
                    try (var dos = new DeflaterOutputStream(comp, deflater)) {
                        dos.write(e.data);
                    }
                    dataToWrite = comp.toByteArray();
                    compSize = dataToWrite.length;
                    method = 8;
                    crcVal = e.crc;
                }

                var cal = Calendar.getInstance();
                cal.setTimeInMillis(e.time);
                int dosTime = (cal.get(Calendar.HOUR_OF_DAY) << 11)
                        | (cal.get(Calendar.MINUTE) << 5)
                        | (cal.get(Calendar.SECOND) >> 1);
                int dosDate = ((cal.get(Calendar.YEAR) - 1980) << 9)
                        | ((cal.get(Calendar.MONTH) + 1) << 5)
                        | cal.get(Calendar.DAY_OF_MONTH);
                byte[] nameBytes = e.name.getBytes("UTF-8");

                out.write(new byte[]{0x50, 0x4b, 0x03, 0x04});
                writeLeShort(out, 20);
                writeLeShort(out, 0);
                writeLeShort(out, method);
                writeLeShort(out, dosTime);
                writeLeShort(out, dosDate);
                writeLeInt(out, crcVal);
                writeLeInt(out, compSize);
                writeLeInt(out, e.data.length);
                writeLeShort(out, nameBytes.length);
                writeLeShort(out, 0);
                out.write(nameBytes);
                out.write(dataToWrite);

                cd.write(new byte[]{0x50, 0x4b, 0x01, 0x02});
                cd.write(packLeShort(0));
                cd.write(packLeShort(20));
                cd.write(packLeShort(0));
                cd.write(packLeShort(method));
                cd.write(packLeShort(dosTime));
                cd.write(packLeShort(dosDate));
                cd.write(packLeInt(crcVal));
                cd.write(packLeInt(compSize));
                cd.write(packLeInt(e.data.length));
                cd.write(packLeShort(nameBytes.length));
                cd.write(packLeShort(0));
                cd.write(packLeShort(0));
                cd.write(packLeShort(0));
                cd.write(packLeShort(0));
                cd.write(packLeInt(0));
                cd.write(packLeInt(localOffset));
                cd.write(nameBytes);

                localOffset += 30 + nameBytes.length + dataToWrite.length;
            }

            byte[] cdBytes = cd.toByteArray();
            out.write(cdBytes);
            out.write(new byte[]{0x50, 0x4b, 0x05, 0x06});
            writeLeShort(out, 0);
            writeLeShort(out, 0);
            writeLeShort(out, entries.size());
            writeLeShort(out, entries.size());
            writeLeInt(out, cdBytes.length);
            writeLeInt(out, localOffset);
            writeLeShort(out, 0);
        }
    }

    private static void writeLeShort(OutputStream out, int v) throws IOException {
        out.write(v & 0xFF);
        out.write((v >> 8) & 0xFF);
    }

    private static void writeLeInt(OutputStream out, long v) throws IOException {
        out.write((int) (v & 0xFF));
        out.write((int) ((v >> 8) & 0xFF));
        out.write((int) ((v >> 16) & 0xFF));
        out.write((int) ((v >> 24) & 0xFF));
    }

    private static byte[] packLeShort(int v) {
        return new byte[]{(byte) (v & 0xFF), (byte) ((v >> 8) & 0xFF)};
    }

    private static byte[] packLeInt(long v) {
        return new byte[]{(byte) (v & 0xFF), (byte) ((v >> 8) & 0xFF), (byte) ((v >> 16) & 0xFF), (byte) ((v >> 24) & 0xFF)};
    }
}
