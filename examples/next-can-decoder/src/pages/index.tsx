import dynamic from 'next/dynamic';
import React, { useState } from 'react';

const DynamicFileUploader = dynamic(
  () => import('@/components/FileUpload'),
  { ssr: false }
);

const Home = () => {
  const [specFile, setSpecFile] = useState<File | null>(null);

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setSpecFile(file);
    }
  };
  return (
    <div>
      <h1>Upload your annex</h1>
      <input type="file" onChange={handleFileChange} />
      <h1>Upload your can log file</h1>
      <DynamicFileUploader specFile={specFile}/>
    </div>
  );
};

export default Home;
